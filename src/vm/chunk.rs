use super::op::Op;

pub struct ByteCodeChunk {
    pub content: Vec<u8>,
    pub strings: Vec<String>,
}

impl ByteCodeChunk {
    pub fn new() -> Self {
        ByteCodeChunk {
            content: Vec::new(),
            strings: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.content.len()
    }

    pub fn write(&mut self, bytes: &[u8]) {
        self.content.extend(bytes)
    }

    pub fn read_i64(&self, offset: usize) -> i64 {
        let slice = &self.content[offset..offset + 8];
        i64::from_ne_bytes(slice.try_into().expect("read_i64 incorrect slice size"))
    }

    pub fn write_i64(&mut self, v: i64) {
        self.write(&i64::to_ne_bytes(v))
    }

    pub fn write_usize(&mut self, v: usize) {
        self.write(&usize::to_ne_bytes(v))
    }

    pub fn read_f64(&self, offset: usize) -> f64 {
        let slice = &self.content[offset..offset + 8];
        f64::from_ne_bytes(slice.try_into().expect("read_f64 incorrect slice size"))
    }

    pub fn write_f64(&mut self, v: f64) {
        self.write(&f64::to_ne_bytes(v))
    }

    pub fn read_op(&self, offset: usize) -> Op {
        Op::from(self.content[offset])
    }

    pub fn write_op(&mut self, op: Op) {
        self.write(&[op as u8])
    }

    pub fn write_bool(&mut self, v: bool) {
        let a = if v { 1 as u8 } else { 0 as u8 };
        self.write(&[a]);
    }

    pub fn add_string(&mut self, text: String) -> usize {
        match self.strings.iter().position(|s| s.eq(&text)) {
            Some(id) => id,
            None => {
                self.strings.push(text);
                self.strings.len() - 1
            }
        }
    }

    pub fn get_string(&self, id: usize) -> &str {
        self.strings[id].as_str()
    }

    pub fn display(&self) -> String {
        let mut output = String::new();

        output += format!("chunk: 000000 - {:06}\n", self.content.len()).as_str();

        output += "  strings:\n";

        for (i, s) in self.strings.iter().enumerate() {
            output += format!("    {:03} {}\n", i, s).as_str();
        }

        output += "\n  code:\n";

        let mut offset = 0;
        while offset < self.len() {
            let (new_offset, text) = self.disassemble(offset);
            offset = new_offset;
            output += "    ";
            output += text.as_str();
            output += "\n";
        }

        output += "\n";

        output
    }
}
