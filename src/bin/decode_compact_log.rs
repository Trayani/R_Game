/// Decoder for compact binary action logs
///
/// Reads .bin files produced by CompactLogWriter and outputs human-readable format
/// optimized for Claude Code analysis

use std::env;
use std::fs;
use std::io;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <action_log.bin>", args[0]);
        eprintln!("Decodes compact binary action log to human-readable format");
        std::process::exit(1);
    }

    let filename = &args[1];
    let data = fs::read(filename)?;

    println!("=== Compact Action Log: {} ===", filename);
    println!("File size: {} bytes\n", data.len());

    let mut reader = CompactLogReader::new(&data);
    let mut event_count = 0;

    while let Some(event) = reader.read_event() {
        event_count += 1;
        println!("{}", event);
    }

    println!("\n=== Summary ===");
    println!("Total events: {}", event_count);
    println!("Average bytes per event: {:.2}", data.len() as f64 / event_count as f64);

    Ok(())
}

struct CompactLogReader<'a> {
    data: &'a [u8],
    pos: usize,
    last_timestamp: u64,
}

impl<'a> CompactLogReader<'a> {
    fn new(data: &'a [u8]) -> Self {
        CompactLogReader {
            data,
            pos: 0,
            last_timestamp: 0,
        }
    }

    fn read_event(&mut self) -> Option<String> {
        if self.pos >= self.data.len() {
            return None;
        }

        // Read timestamp delta
        let delta = self.read_varint().ok()?;
        self.last_timestamp += delta;

        // Read action type
        let type_byte = self.read_u8().ok()?;
        let action_type = type_byte & 0x7F;
        let phase = if type_byte & 0x80 != 0 { "END" } else { "START" };

        let action_str = match action_type {
            1 => format!("SetBlocked({}, {})", self.read_i32().ok()?, self.read_i32().ok()?),
            2 => format!("SetFree({}, {})", self.read_i32().ok()?, self.read_i32().ok()?),
            3 => format!("ToggleCell({}, {})", self.read_i32().ok()?, self.read_i32().ok()?),
            4 => {
                let x = self.read_i32().ok()?;
                let y = self.read_i32().ok()?;
                let flags = self.read_u8().ok()?;
                let messy_x = flags & 1 != 0;
                let messy_y = flags & 2 != 0;
                format!("MoveObserver({},{} mx={} my={})", x, y, messy_x, messy_y)
            }
            5 => "ToggleMessyX".to_string(),
            6 => "ToggleMessyY".to_string(),
            7 => format!("SetObserverDest({}, {})", self.read_i32().ok()?, self.read_i32().ok()?),
            8 => format!("SpawnActor({:.1}, {:.1})", self.read_f32().ok()?, self.read_f32().ok()?),
            9 => {
                let x = self.read_i32().ok()?;
                let y = self.read_i32().ok()?;
                let count = self.read_varint().ok()?;
                format!("SetActorDest({},{} actors={})", x, y, count)
            }
            10 => format!("PasteGrid({}x{})", self.read_i32().ok()?, self.read_i32().ok()?),
            11 => {
                let aid = self.read_varint().ok()?;
                let cx = self.read_i32().ok()?;
                let cy = self.read_i32().ok()?;
                let cid = self.read_i32().ok()?;
                format!("ActorStartMove(A{} @cell({},{})={})", aid, cx, cy, cid)
            }
            12 => {
                let aid = self.read_varint().ok()?;
                let cx = self.read_i32().ok()?;
                let cy = self.read_i32().ok()?;
                let cid = self.read_i32().ok()?;
                let ncx = self.read_i32().ok()?;
                let ncy = self.read_i32().ok()?;
                let ncid = self.read_i32().ok()?;
                format!("ActorWaypoint(A{} reached({},{})={} next({},{})={})",
                    aid, cx, cy, cid, ncx, ncy, ncid)
            }
            13 => {
                let aid = self.read_varint().ok()?;
                let cx = self.read_i32().ok()?;
                let cy = self.read_i32().ok()?;
                let cid = self.read_i32().ok()?;
                format!("ActorReachedDest(A{} @({},{})={})", aid, cx, cy, cid)
            }
            14 => {
                let ox = self.read_i32().ok()?;
                let oy = self.read_i32().ok()?;
                let flags = self.read_u8().ok()?;
                let total = self.read_varint().ok()?;
                let interesting = self.read_varint().ok()?;
                let messy_x = flags & 1 != 0;
                let messy_y = flags & 2 != 0;
                format!("CalcCorners(obs=({},{}) mx={} my={} total={} interesting={})",
                    ox, oy, messy_x, messy_y, total, interesting)
            }
            15 => {
                let fx = self.read_i32().ok()?;
                let fy = self.read_i32().ok()?;
                let tx = self.read_i32().ok()?;
                let ty = self.read_i32().ok()?;
                let flags = self.read_u8().ok()?;
                let len = self.read_varint().ok()?;
                let messy_x = flags & 1 != 0;
                let messy_y = flags & 2 != 0;
                let success = flags & 4 != 0;
                format!("CalcPath(({},{})→({},{}) mx={} my={} len={} ok={})",
                    fx, fy, tx, ty, messy_x, messy_y, len, success)
            }
            16 => {
                let aid = self.read_varint().ok()?;
                let fx = self.read_f32().ok()?;
                let fy = self.read_f32().ok()?;
                let blocker = self.read_varint().ok()?;
                format!("ActorBlocked(A{} @({:.1},{:.1}) by=A{})", aid, fx, fy, blocker)
            }
            _ => format!("Unknown(type={})", action_type),
        };

        Some(format!("[{:6}ms] {} {}", self.last_timestamp, phase, action_str))
    }

    fn read_u8(&mut self) -> io::Result<u8> {
        if self.pos >= self.data.len() {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "end of data"));
        }
        let val = self.data[self.pos];
        self.pos += 1;
        Ok(val)
    }

    fn read_varint(&mut self) -> io::Result<u64> {
        let mut result = 0u64;
        let mut shift = 0;

        loop {
            let byte = self.read_u8()?;
            result |= ((byte & 0x7F) as u64) << shift;
            if byte & 0x80 == 0 {
                break;
            }
            shift += 7;
        }

        Ok(result)
    }

    fn read_i32(&mut self) -> io::Result<i32> {
        let encoded = self.read_varint()?;
        // ZigZag decode
        let decoded = ((encoded >> 1) as i32) ^ (-((encoded & 1) as i32));
        Ok(decoded)
    }

    fn read_f32(&mut self) -> io::Result<f32> {
        if self.pos + 4 > self.data.len() {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "not enough data for f32"));
        }
        let bytes = [
            self.data[self.pos],
            self.data[self.pos + 1],
            self.data[self.pos + 2],
            self.data[self.pos + 3],
        ];
        self.pos += 4;
        Ok(f32::from_le_bytes(bytes))
    }
}
