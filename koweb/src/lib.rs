use wasm_bindgen::prelude::*;
//parses strings into commands which are then either
// use kontroli::error::SignatureError;
// use kontroli::rc::{Intro, Rule, Signature, Typing};
// use kontroli::scope::{Command, Symbols};
// use kontroli::error::Error;
// use kontroli::error::SymbolsError;
use byte_unit::{Byte, ByteError};
use js_sys::{Error as JsError, JsString, Reflect};
use kocheck::{parse, seq, Error, Event, Opt};
use log::{info, trace, warn, Level};
use serde::{Deserialize, Serialize};

// use crate::itertools::Itertools;

pub mod lazy_fetch;
pub mod parse_make;

//not sure what wee alloc does
// #[cfg(feature = "wee_alloc")]
// #[global_allocator]
// static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

extern crate console_error_panic_hook;
use std::panic;

fn init_console_wasm_debug() {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen]
    fn alert(s: &str);

    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

static mut test: i32 = 0;

#[wasm_bindgen]
pub fn get_string_js(string_js: String) -> String {
    return string_js;
}

#[wasm_bindgen()]
pub fn increment_test() {
    unsafe {
        // alert(Prog::get_piece_to_koweb_static().as_str());
        // alert(Test::test_text().as_str());
        // alert(call_exported_rust_func().as_str());
        // alert(Test2::test_text2().as_str()); MIME TYPE ISSUE AAA
        // let value = js_sys::Reflect::get(&target, &property_key).expect("reflect failed");
        alert(format!("test : {}", test).as_str());
        test += 1;
    }
}

fn get_text_from_editor() -> Result<String, ()> {
    let window = web_sys::window().expect("no window found");
    // let editor = window.editor();
    let object = match Reflect::get(&window, &js_sys::JsString::from("editor")) {
        Ok(value) if value.is_object() => Ok(value),
        _ => Err("Window object doesn't have a suitable property"),
    }
    .unwrap();

    let method: js_sys::Function = match Reflect::get(&object, &js_sys::JsString::from("getValue"))
    {
        Ok(value) if value.is_function() => {
            // wasm_bindgen::JsValue => js_sys::Function
            Ok(value.into())
        }
        _ => Err("object does not have the specified method"),
    }
    .unwrap();

    let arguments = js_sys::Array::new();

    match Reflect::apply(&method, &object, &arguments) {
        Ok(result) => {
            info!("Applied method successfully.");
            info!("This is the result {:?}", result);
            return Ok(result.as_string().unwrap()); //find how to change JsValue to string
        }
        Err(error) => {
            info!("Attempt to apply method failed.");
        }
    }
    Err(())
}

fn produce_from_js<'a>(
    cmds_from_js: &'a String,
    opt: &Opt,
) -> impl Iterator<Item = Result<Event, Error>> + 'a {
    // let module = std::iter::once(Ok(Event::Module(vec!["js".to_string()]))); //TODO is this needed ?
    return parse(cmds_from_js.as_bytes(), opt).map(|cmd| cmd.map(Event::Command));
    // cmds
    // module.chain(cmds)
}

fn string_to_static_str(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

fn write_to_webpage(event: &kocheck::Event) {
    //i need to match here on the cocheck enum ??
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let body = document.body().expect("document should have a body");
    // let val = document.get_element_by_id("console").unwrap();
    let div = document.create_element("div").unwrap();
    let text = document.create_element("p").unwrap();
    let lambda_span = document.create_element("span").unwrap();
    lambda_span.set_class_name("lambda");
    lambda_span.set_text_content(Some("λ> "));
    text.set_class_name("line");
    text.set_text_content(Some(format!("{}", event).as_str()));
    div.set_class_name("prompt");
    div.append_child(&lambda_span);
    div.append_child(&text);
    let output = document.get_element_by_id("output").unwrap();
    output.set_class_name("display_output");
    output.append_child(&div).unwrap();
}

//temporary i'll use the definition from bin maybe
pub fn flatten_nested_results<O, I, T, E>(outer: O) -> impl Iterator<Item = Result<T, E>>
where
    O: Iterator<Item = Result<I, E>>,
    I: Iterator<Item = Result<T, E>>,
{
    outer.flat_map(|inner_result| {
        let (v, r) = match inner_result {
            Ok(v) => (Some(v), None),
            Err(e) => (None, Some(Err(e))),
        };
        v.into_iter().flatten().chain(r)
    })
}

fn print_iterator<I>(iter: &mut I)
where
    I: Iterator<Item = Result<Event, Error>>,
{
    for element in iter {
        match element {
            Result::Ok(Event) => log(format!("{}", Event).as_str()),
            Result::Err(Error) => log(format!("something went wrong : {:?}", Error).as_str()),
        }
    }
}

#[wasm_bindgen]
pub fn run_test(cmds_from_js: String, eta: bool, no_scope: bool, no_infer: bool, no_check: bool) {
    // let mut f = File::open("foo.txt")?;
    // let mut buffer = [0; 10];

    console_log::init_with_level(Level::Trace);
    init_console_wasm_debug();
    // alert(cmds_from_js.as_str());

    let program_text = get_text_from_editor().unwrap();

    // let static_cmds_str = string_to_static_str(cmds_from_js);

    let opt = Opt {
        eta,
        no_scope,
        no_infer,
        no_check,
        buffer: Byte::from_str("64MB").unwrap(),
        channel_capacity: None,
        jobs: None,
        files: vec![],
    };

    let iter = produce_from_js(&program_text, &opt);

    let mut iter = Box::new(iter).inspect(|r| r.iter().for_each(|event| write_to_webpage(event)));

    seq::consume(iter, &opt).expect("something went wrong in the consume");

    // Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Program {
    name: String,
    dependency: Vec<String>,
    dependency_url_list: Vec<String>,
    raw_url: String,
}

//TODO split in multiple files better after
pub mod fetch_buffer;

#[wasm_bindgen]
pub async fn run_multiple(
    programs: JsValue,
    module_to_run: String,
    eta: bool,
    no_scope: bool,
    no_infer: bool,
    no_check: bool,
) {
    console_log::init_with_level(Level::Trace);
    init_console_wasm_debug();
    let vec_of_programs: Vec<Program> = programs.into_serde().unwrap();
    info!(
        "this is the program we want to run : {} ",
        module_to_run.as_str()
    );
    // info!("PROGRAM LIST IN RUST : {:?}", vec_of_programs);

    for program in vec_of_programs {
        if program.name == module_to_run {
            info!("This is all the program info -> {:?}", program);
            info!("Name of the Program we want to run -> {}", module_to_run);
            for file in program.dependency_url_list {
                info!("running file => {}", file);
                let res = lazy_fetch::get_program_text(&file).await;
                info!("this is what we got from the fetching {:?}", res);
                let test_string = String::from_utf8(res.unwrap().into_inner()).unwrap();
                info!(
                    "this is what we got from the fetching turned into a string: {}",
                    test_string
                );
                let opt = Opt {
                    eta,
                    no_scope,
                    no_infer,
                    no_check,
                    buffer: Byte::from_str("64MB").unwrap(),
                    channel_capacity: None,
                    jobs: None,
                    files: vec![],
                };
                //TODO right now things are done here but i just want to pass the program that we wan to run to a produce fetch or something
                //so i don't really want to use produce js
                let iter = produce_from_js(&test_string, &opt);
                //TODO module names and separation in the output
                let mut iter =
                    Box::new(iter).inspect(|r| r.iter().for_each(|event| write_to_webpage(event)));
                seq::consume(iter, &opt).expect("something went wrong in the consume");
                info!("function is finished 0-0");
            }
        }
    }
}

use fetch_buffer::FetchBuffer;
use kontroli::error::Error as KoError;
use kontroli::parse::Command;
use nom::error::VerboseError;
use std::convert::TryInto;
use std::io::Read;

//but one fill might be too large for the buffer if its a very large file so
//i need to fill what i can and then check if i go to the end of the file being fetched
//if i did and there are other files

//do i need to change next no not really right

async fn produce_from_fetch(dependency_url_list: Vec<String>, opt: &Opt) {
    //create a parse buffer here with the data
    use kontroli::parse::{opt_lex, phrase, Parse, Parser};
    let parse: fn(&[u8]) -> Parse<_> = |i| opt_lex(phrase(Command::parse))(i);

    //TODO let's try and do everything here and then we will see

    let mut pb = FetchBuffer {
        buf: circular::Buffer::with_capacity(opt.buffer.get_bytes().try_into().unwrap()),
        read_r: std::io::Cursor::new(vec![]),
        read: std::io::Cursor::new(vec![]),
        parse,
        fail: |_: nom::Err<VerboseError<&[u8]>>| Error::Ko(KoError::Parse),
        urls: dependency_url_list,
        file_counter: 0,
    };

    //TODO think maybe i don't need to do anything in the next ?
    //but im pretty sure that i do or maybe
    //right now fill is called here once and in next but it looks like next is called implicitly because i have not found any place
    //where it is called explicitly
    pb.fill().await.unwrap();
    pb.map(|entry| entry.transpose()).flatten(); //on this produce called .map(|cmd| cmd.map(Event::Command));
                                                 //     // cmds
                                                 //     // module.chain(cmds)
}

fn parse_fetch() {}

//produce js calls parse which makes the parse buffer that parses the string

//like i could just pass the list of urls in here
// fn produce_from_js<'a>(
//     cmds_from_js: &'a String,
//     opt: &Opt,
// ) -> impl Iterator<Item = Result<Event, Error>> + 'a {
//     // let module = std::iter::once(Ok(Event::Module(vec!["js".to_string()]))); //TODO is this needed ?
//     return parse(cmds_from_js.as_bytes(), opt).map(|cmd| cmd.map(Event::Command));
//     // cmds
//     // module.chain(cmds)
// }

// pub fn parse<R: Read>(read: R, opt: &Opt) -> impl Iterator<Item = Result<Command, Error>> {
//     use kontroli::parse::{opt_lex, phrase, Parse, Parser};
//     let parse: fn(&[u8]) -> Parse<_> = |i| opt_lex(phrase(Command::parse))(i);
//     let mut pb = ParseBuffer {
//         buf: circular::Buffer::with_capacity(opt.buffer.get_bytes().try_into().unwrap()),
//         read,
//         parse,
//         fail: |_: nom::Err<VerboseError<&[u8]>>| Error::Ko(KoError::Parse),
//     };
//     pb.fill().unwrap();
//     // consider only non-whitespace entries
//     pb.map(|entry| entry.transpose()).flatten()
// }

//TODO
//i have to make a parse buffer now uh and my own parse function then passing the list of the urls instead of the file
//how come i am not getting an error when i run a single file

//ok so the thing that i am fetching is not actually the right thing
//i am fetching the first one in the dependency
//so i am actually running sttfa.dk right now that makes sense lets try to run something that i can't actually run

// [2021-05-18T11:23:16Z INFO  kocheck] Open module /bigops
// [2021-05-18T11:23:16Z INFO  kocheck] Introduce symbol sameF_upto
// Error: Ko(Scope(UndeclaredSymbol("sttfa.etap")))
//so this is what i get when i run it with kocheckj why don't i get this when i run it with koweb
