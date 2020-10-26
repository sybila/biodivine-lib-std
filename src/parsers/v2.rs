use regex::Regex;

#[derive(Clone, Debug)]
pub struct Token {
    rule: String,
    starts_at: usize,
    data: String,
}

impl Token {
    pub fn new(rule: &str, data: &str, position: usize, length: usize) -> Token {
        return Token {
            rule: rule.to_string(),
            starts_at: position,
            data: (&data[position..position + length]).to_string(),
        };
    }
}

#[derive(Clone, Debug)]
pub struct Error {
    starts_at: Option<usize>,
    ends_at: Option<usize>,
    message: String,
}

pub trait Tokenizer<S> {
    fn empty_state(&self) -> S;

    fn scan_token(&self, state: &mut S, position: &mut usize, data: &str) -> Option<Token>;

    fn scan(&self, data: &str) -> Result<Vec<Token>, Error> {
        let mut state = self.empty_state();
        let mut position: usize = 0;
        let mut result = Vec::new();
        while position < data.len() {
            if let Some(token) = self.scan_token(&mut state, &mut position, data) {
                result.push(token);
            } else {
                panic!("Failed: {:?}", &data[position..]);
            }
        }
        return Ok(result);
    }
}

type TokenizerBox<S> = Box<dyn Tokenizer<S>>;
type StaticTokenizerBox = Box<dyn StaticTokenizer>;

pub trait StaticTokenizer: Tokenizer<()> {
    fn scan_token_static(&self, position: &mut usize, data: &str) -> Option<Token>;
}

impl<T: StaticTokenizer> Tokenizer<()> for T {
    fn empty_state(&self) -> () {
        return ();
    }

    fn scan_token(&self, _: &mut (), position: &mut usize, data: &str) -> Option<Token> {
        return self.scan_token_static(position, data);
    }
}

pub struct SkipTokenizer<S, V> {
    skip: Box<dyn Tokenizer<S>>,
    valid: Box<dyn Tokenizer<V>>,
}


pub struct SequenceTokenizer {
    inner: Vec<Box<dyn StaticTokenizer>>,
}
pub struct NestedGroupsTokenizer {
    open: Box<dyn StaticTokenizer>,
    close: Box<dyn StaticTokenizer>,
    body: Box<dyn StaticTokenizer>,
}


impl<S, V> Tokenizer<(S, V)> for SkipTokenizer<S, V> {
    fn empty_state(&self) -> (S, V) {
        return (self.skip.empty_state(), self.valid.empty_state());
    }

    fn scan_token(&self, state: &mut (S, V), position: &mut usize, data: &str) -> Option<Token> {
        while let Some(t) = self.skip.scan_token(&mut state.0, position, data) {}
        return self.valid.scan_token(&mut state.1, position, data);
    }
}
