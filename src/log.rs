use termimad::crossterm::style::{Color::*, Attribute::*};
use termimad::*;
use chrono::Local;
use crate::THREAD_POOL_SIZE;

pub enum LogLevel{
    Info,
    Warn,
    Error,
    Debug,
}
impl LogLevel{
    pub fn as_str(&self) -> &'static str{
        match self {
            LogLevel::Info => "Info",
            LogLevel::Warn => "Warn",
            LogLevel::Error => "Error",
            LogLevel::Debug => "Debug",
        }
    }
}

#[derive(Clone)]
pub struct ConsoleContext{
    pub content:Vec<String>,
    pub header:String,
    pub workers_status:Vec<i32>,
    pub skin:MadSkin,
}

pub fn create_console() -> ConsoleContext{
    let mut skin = MadSkin::default();
    // let's decide bold is in light gray
    skin.bold.set_fg(Green);
    skin.italic.set_fg(Red);
    skin.strikeout.set_fg(Yellow);
    skin.inline_code.set_fg(Grey);

    ConsoleContext{
        content:Vec::new(),
        header: "".to_string(),
        workers_status: vec![0; THREAD_POOL_SIZE],
        skin
    }
}

pub fn update_header(ctx: &mut ConsoleContext){
    //0 - green - responded sucessfully
    //1 - red - failed request
    //2 - yellow - processing
    //3 - grey - waiting
    //4.. - hollow - dead

    let mut bulbs = vec![];
    for i in 0..THREAD_POOL_SIZE {
        let char = match ctx.workers_status[i] {
            0=>{
                "**●**"
            },
            1=>{
                "*●*"
            },
            2=>{
                "~~●~~"
            },
            3=>{
                "`●`"
            },
            _ => {
                "○"
            }
        };
        bulbs.push(char);
    }
    ctx.header = format!("Workers ({THREAD_POOL_SIZE}): ")+bulbs.join(" ").as_str();
}
pub fn render_console(mut ctx: &mut ConsoleContext){
    update_header(&mut ctx);
    let area = Area::new(0, 0, get_size().0, get_size().1);
    let content = ctx.header.clone() + "\n" +ctx.content.join("\n").as_str();
    let mut view = MadView::from(content, area, ctx.skin.clone());
    view.write().unwrap()
}

pub fn format_message(message: String, level: LogLevel,worker:i32)->String{
    let date = Local::now().format("%Y-%m-%d %H:%M:%S");
    format!("{0} : [{1}] @{2} - {3}\n", date, level.as_str(),worker, message)
}

pub fn log_message(message: String, level: LogLevel,worker:i32,ctx:&mut ConsoleContext){
    let formatted_message = format_message(message, level,worker);
    ctx.content.push(formatted_message);
}

pub fn info(message: &str,worker:i32, ctx: &mut ConsoleContext){
    log_message(message.to_string(), LogLevel::Info,worker,ctx);
}
pub fn warn(message: &str,worker:i32,ctx: &mut ConsoleContext){
    log_message(message.to_string(), LogLevel::Warn,worker,ctx);
}
pub fn error(message: &str,worker:i32,ctx: &mut ConsoleContext){
    log_message(message.to_string(), LogLevel::Error,worker,ctx);
}
pub fn debug(message: &str,worker:i32,ctx: &mut ConsoleContext){
    log_message(message.to_string(), LogLevel::Debug,worker,ctx);
}

pub fn get_size() -> (u16, u16) {
    terminal_size()
}