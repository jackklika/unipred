use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Tick(pub i32); // 4 decimals: 1 tick = $0.0001

impl Tick {
    pub const SCALE: i32 = 10_000;

    pub fn from_dollars_round_down(d: &str) -> Option<Self> {
        // parse kalshi dollar format, ie "0.2000"
        let value: f64 = d.parse().ok()?;
        let ticks = (value * Self::SCALE as f64).floor() as i32;
        Some(Tick(ticks))
    }

    pub fn to_dollars_string(self) -> String {
        let abs = self.0.abs() as i32;
        let int = abs / Self::SCALE;
        let frac = abs % Self::SCALE;
        if self.0 < 0 {
            format!("-{int}.{frac:04}")
        } else {
            format!("{int}.{frac:04}")
        }
    }
}

pub fn de_tick_levels<'de, D>(de: D) -> Result<Option<Vec<(Tick, i32)>>, D::Error>
where
    D: Deserializer<'de>,
{
    // Accept either string or number for price, e.g. ["0.2000", 350] or [0.2, 350]
    let raw: Option<Vec<(serde_json::Value, i32)>> = Option::deserialize(de)?;
    match raw {
        None => Ok(None),
        Some(items) => {
            let mut out = Vec::with_capacity(items.len());
            for (price_v, qty) in items {
                let s = match price_v {
                    serde_json::Value::String(s) => s,
                    serde_json::Value::Number(n) => n
                        .to_string(), // safe: we'll parse via Tick::from_dollars_round_down
                    _ => return Err(serde::de::Error::custom("price must be string or number")),
                };
                let tick = Tick::from_dollars_round_down(&s)
                    .ok_or_else(|| serde::de::Error::custom("invalid price format"))?;
                out.push((tick, qty));
            }
            Ok(Some(out))
        }
    }
}

pub fn se_tick_levels<S>(levels: &Option<Vec<(Tick, i32)>>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match levels {
        None => s.serialize_none(),
        Some(v) => {
            // Serialize as [( "0.2000", 350 ), ...]
            let rendered: Vec<(String, i32)> = v
                .iter()
                .map(|(t, q)| (t.to_dollars_string(), *q))
                .collect();
            rendered.serialize(s)
        }
    }
}
