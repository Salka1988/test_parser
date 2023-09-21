use serde::de::DeserializeSeed;
use serde_json::Value;

/// A simplified state passed to and returned from the serialization.
#[derive(Debug, Default)]
struct Stats {
    records_skipped: usize,
}

/// Models the input data; `Documents` is just a vector of JSON values,
/// but it is its own type to allow custom deserialization
#[derive(Debug)]
struct MyData {
    documents: Vec<Value>,
    journal: Value,
}

struct MyDataDeserializer<'a> {
    state: &'a mut Stats,
}

/// Top-level seeded deserializer only so I can plumb the state through
impl<'de> DeserializeSeed<'de> for MyDataDeserializer<'_> {
    type Value = MyData;

    fn deserialize<D>(mut self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde::Deserializer<'de>,
    {
        let visitor = MyDataVisitor(&mut self.state);
        let docs = deserializer.deserialize_map(visitor)?;
        Ok(docs)
    }
}

struct MyDataVisitor<'a>(&'a mut Stats);

impl<'de> serde::de::Visitor<'de> for MyDataVisitor<'_> {
    type Value = MyData;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "a map")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::MapAccess<'de>,
    {
        let mut documents = Vec::new();
        let mut journal = Value::Null;

        while let Some(key) = map.next_key::<String>()? {
            println!("Got key = {key}");
            match &key[..] {
                "documents" => {
                    // Not sure how to handle the next value in a streaming manner
                    documents = map.next_value()?;
                }

                "journal" => journal = map.next_value()?,
                _ => panic!("Unexpected key '{key}'"),
            }
        }

        Ok(MyData { documents, journal })
    }
}

struct DocumentDeserializer<'a> {
    state: &'a mut Stats,
}

impl<'de> DeserializeSeed<'de> for DocumentDeserializer<'_> {
    type Value = Vec<Value>;

    fn deserialize<D>(mut self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde::Deserializer<'de>,
    {
        let visitor = DocumentVisitor(&mut self.state);
        let documents = deserializer.deserialize_seq(visitor)?;
        Ok(documents)
    }
}

struct DocumentVisitor<'a>(&'a mut Stats);

impl<'de> serde::de::Visitor<'de> for DocumentVisitor<'_> {
    type Value = Vec<Value>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "a list")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::SeqAccess<'de>,
    {
        let mut agg_map = serde_json::Map::new();

        while let Some(item) = seq.next_element()? {
            // If `item` isn't a JSON object, we'll skip it:
            let Value::Object(map) = item else { continue };

            // Get the first element, assuming we have some
            let (k, v) = match map.into_iter().next() {
                Some(kv) => kv,
                None => continue,
            };

            // Ignore any null values; aggregate everything into a single map
            if v == Value::Null {
                self.0.records_skipped += 1;
                continue;
            } else {
                println!("Keeping {k}={v}");
                agg_map.insert(k, v);
            }
        }
        let values = Value::Object(agg_map);
        println!("Final value is {values}");

        Ok(vec![values])
    }
}

fn main() {
    let fh = std::fs::File::open("input.json").unwrap();
    let buf = std::io::BufReader::new(fh);
    let read = serde_json::de::IoRead::new(buf);

    let mut state = Stats::default();
    let mut deserializer = serde_json::Deserializer::new(read);

    let mydata = MyDataDeserializer { state: &mut state }
        .deserialize(&mut deserializer)
        .unwrap();

    println!("{mydata:?}");
}