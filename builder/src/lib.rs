use wasm_actions::get_input;
use wasm_actions_core::error::Error;

pub trait ActionInput {
    fn parse() -> Result<Self, Error> where Self: Sized;
}

pub trait ActionOutput {
    fn parse() -> Result<Option<Self>, Error> where Self: Sized;
    async fn save(self) -> Result<(), Error>;
}

pub trait Action<I: ActionInput, O: ActionOutput> {
    fn parse_input() -> Result<I, Error> {
        I::parse()
    }

    fn parse_state() -> Result<Option<O>, Error> {
        O::parse()
    }

    async fn main(input: I) -> Result<O, Error>;

    async fn post(_input: I, _state: O) -> Result<(), Error> {
        Ok(())
    }
}

pub struct InputParser;
