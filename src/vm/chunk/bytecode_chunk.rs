use crate::{vm::op::Op, vm::value::ivalue};

#[derive(Debug, Clone)]
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

    #[inline(always)]
    pub fn write_op(&mut self, v: Op) {
        self.content.extend(&[v as u8])
    }

    #[inline(always)]
    pub fn write_ivalue(&mut self, v: ivalue) {
        self.content.extend(&ivalue::to_ne_bytes(v))
    }

    #[inline(always)]
    pub fn write_usize(&mut self, v: usize) {
        self.content.extend(&usize::to_ne_bytes(v))
    }

    pub fn add_string(&mut self, text: String) -> ivalue {
        match self.strings.iter().position(|s| s.eq(&text)) {
            Some(id) => id as ivalue,
            None => {
                self.strings.push(text);
                (self.strings.len() - 1) as ivalue
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

        let dis = match self.disassemble() {
            Ok(s) => s,
            Err(e) => format!("   disassembly failed: {:?}", e),
        };

        output.push_str(dis.as_str());
        output += "\n";

        output
    }
}
