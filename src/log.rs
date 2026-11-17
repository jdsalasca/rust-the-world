use std::collections::VecDeque;

pub const MAX_LINES: usize = 5;

pub struct EventLog {
    lines: VecDeque<String>,
}

impl EventLog {
    pub fn new() -> EventLog {
        EventLog { lines: VecDeque::new() }
    }

    pub fn push(&mut self, line: String) {
        while self.lines.len() >= MAX_LINES {
            self.lines.pop_front();
        }
        self.lines.push_back(line);
    }

    pub fn lines(&self) -> &VecDeque<String> {
        &self.lines
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_keeps_last_five() {
        let mut log = EventLog::new();
        for index in 0..8 {
            log.push(format!("linea {index}"));
        }
        assert_eq!(log.lines().len(), 5);
        assert_eq!(log.lines()[0], "linea 3");
        assert_eq!(log.lines()[4], "linea 7");
    }
}
