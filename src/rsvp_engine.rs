use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct Word {
    pub text: String,
    pub orp_index: usize,
    pub display_time: Duration,
}

impl Word {
    pub fn new(text: String, wpm: u32) -> Self {
        let orp_index = Self::calculate_orp(&text);
        let base_duration = Duration::from_secs_f32(60.0 / wpm as f32);

        // Adjust display time based on word length and punctuation
        let length_factor = 1.0 + (text.len() as f32 - 5.0) * 0.03;
        let punctuation_factor = if text.contains(&['.', '!', '?', ';'][..]) {
            1.4
        } else if text.contains(',') {
            1.15
        } else {
            1.0
        };

        let display_time = base_duration.mul_f32(length_factor.max(0.8) * punctuation_factor);

        Self {
            text,
            orp_index,
            display_time,
        }
    }

    fn calculate_orp(text: &str) -> usize {
        let len = text.len();
        match len {
            1..=3 => 0,
            4..=5 => 1,
            6..=9 => 2,
            10..=13 => 3,
            _ => 4.min(len - 1),
        }
    }

    pub fn get_parts(&self) -> (String, char, String) {
        let chars: Vec<char> = self.text.chars().collect();

        if self.orp_index >= chars.len() {
            return (self.text.clone(), ' ', String::new());
        }

        let before = chars[..self.orp_index].iter().collect();
        let focus = chars[self.orp_index];
        let after = chars[self.orp_index + 1..].iter().collect();

        (before, focus, after)
    }
}

pub struct RSVPEngine {
    words: Vec<Word>,
    current_index: usize,
    last_update: Instant,
    is_paused: bool,
    current_wpm: u32,
    target_wpm: u32,
    start_wpm: u32,
    warmup_words: u32,
}

impl RSVPEngine {
    pub fn new(text: &str, start_wpm: u32, target_wpm: u32, warmup_words: u32) -> Self {
        let words: Vec<Word> = text
            .split_whitespace()
            .map(|w| Word::new(w.to_string(), start_wpm))
            .collect();

        Self {
            words,
            current_index: 0,
            last_update: Instant::now(),
            is_paused: false,
            current_wpm: start_wpm,
            target_wpm,
            start_wpm,
            warmup_words,
        }
    }

    pub fn update(&mut self) -> Option<&Word> {
        if self.is_paused || self.words.is_empty() || self.current_index >= self.words.len() {
            return None;
        }

        // Calculate current WPM based on word count progress
        if self.current_index < self.warmup_words as usize {
            let progress = self.current_index as f32 / self.warmup_words as f32;
            self.current_wpm = self.start_wpm +
                ((self.target_wpm - self.start_wpm) as f32 * progress) as u32;
        } else {
            self.current_wpm = self.target_wpm;
        }

        let now = Instant::now();

        // Calculate display time for current word at current speed
        let base_duration = Duration::from_secs_f32(60.0 / self.current_wpm as f32);
        let current_word = &self.words[self.current_index];
        let length_factor = 1.0 + (current_word.text.len() as f32 - 5.0) * 0.03;
        let punctuation_factor = if current_word.text.contains(&['.', '!', '?', ';'][..]) {
            1.4
        } else if current_word.text.contains(',') {
            1.15
        } else {
            1.0
        };
        let display_time = base_duration.mul_f32(length_factor.max(0.8) * punctuation_factor);

        if now.duration_since(self.last_update) >= display_time {
            self.last_update = now;
            let word = &self.words[self.current_index];
            self.current_index += 1;
            Some(word)
        } else {
            Some(current_word)
        }
    }

    pub fn pause(&mut self) {
        self.is_paused = true;
    }

    pub fn resume(&mut self) {
        self.is_paused = false;
        self.last_update = Instant::now();
    }

    pub fn toggle_pause(&mut self) {
        if self.is_paused {
            self.resume();
        } else {
            self.pause();
        }
    }

    pub fn reset(&mut self) {
        self.current_index = 0;
        self.last_update = Instant::now();
        self.current_wpm = self.start_wpm;
    }

    pub fn adjust_speed(&mut self, delta: i32) {
        let new_wpm = (self.target_wpm as i32 + delta).max(100).min(1200) as u32;
        self.target_wpm = new_wpm;
        self.current_wpm = new_wpm;
    }

    pub fn seek(&mut self, delta: i32) {
        let new_index = (self.current_index as i32 + delta).max(0) as usize;
        self.current_index = new_index.min(self.words.len().saturating_sub(1));
        self.last_update = Instant::now();
    }

    pub fn seek_to(&mut self, index: usize) {
        self.current_index = index.min(self.words.len().saturating_sub(1));
        self.last_update = Instant::now();
    }

    pub fn get_current_index(&self) -> usize {
        self.current_index
    }

    pub fn get_current_word(&self) -> Option<&Word> {
        self.words.get(self.current_index)
    }

    pub fn is_finished(&self) -> bool {
        self.current_index >= self.words.len()
    }

    pub fn get_progress(&self) -> f32 {
        if self.words.is_empty() {
            0.0
        } else {
            self.current_index as f32 / self.words.len() as f32
        }
    }

    pub fn get_current_wpm(&self) -> u32 {
        self.current_wpm
    }
}