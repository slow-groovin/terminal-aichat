use crossterm::{
    style::{Color, SetBackgroundColor},
    cursor, terminal, QueueableCommand,
    ExecutableCommand,
};
use std::{
    collections::VecDeque, io::{stdout, Write}, thread::JoinHandle, time::{Duration, Instant}
};
use tokio::{
    sync::mpsc::{self, Sender, Receiver},
    time::sleep,
};

/// 渲染消息类型
#[derive(Debug)]
pub enum RenderMessage {
    Content(String),
    SetStatus(ResponseStatus),
    Stop,
}

/// 响应状态
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ResponseStatus {
    Responding,
    Done,
    Error,
}

impl ResponseStatus {
    fn as_emoji(&self) -> &'static str {
        match self {
            ResponseStatus::Responding => "⏳",
            ResponseStatus::Done => "✅",
            ResponseStatus::Error => "❌",
        }
    }
}

/// 渲染配置
#[derive(Clone)]
pub struct RenderConfig {
    pub pure: bool,
    pub model_name: String,
    pub prompt_name: String,
    /// 打字机效果的速度（字符/秒）
    pub type_speed: u32,
    /// 状态栏最小刷新间隔（毫秒）
    pub status_refresh_interval: u64,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            pure: false,
            model_name: String::new(),
            prompt_name: String::new(),
            type_speed: 50,  // 默认50字/秒
            status_refresh_interval: 1000,  // 默认1秒
        }
    }
}

/// 渲染器内部状态
struct RenderState {
    /// 待渲染的内容队列
    content_queue: VecDeque<String>,
    /// 当前正在打印的内容
    current_content: String,
    /// 当前内容的打印位置
    current_position: usize,
    /// 上次渲染字符的时间
    last_char_time: Instant,
    /// 上次状态栏更新时间
    last_status_update: Instant,
    /// 当前状态
    status: ResponseStatus,
    /// 开始时间
    start_time: Instant,
    start_row: u16,
}

impl RenderState {
    fn new() -> Self {
        Self {
            content_queue: VecDeque::new(),
            current_content: String::new(),
            current_position: 0,
            last_char_time: Instant::now(),
            last_status_update: Instant::now(),
            status: ResponseStatus::Responding,
            start_time: Instant::now(),
            start_row: cursor::position().unwrap().1
        }
    }

    fn has_more_content(&self) -> bool {
        !self.content_queue.is_empty() || self.current_position < self.current_content.len()
    }
}

/// 响应渲染器
pub struct ResponseRenderer {
    message_tx: Sender<RenderMessage>,
    content_tx: Sender<String>,
    message_handler:Option<tokio::task::JoinHandle<()>>,
    render_handler:Option<tokio::task::JoinHandle<()>>
}

impl ResponseRenderer {
    pub fn new(config: RenderConfig) -> Self {
        let (message_tx, message_rx) = mpsc::channel(100);
        let (content_tx, content_rx) = mpsc::channel(1000);  // 内容队列容量更大
        
        
        // 启动消息处理任务
        let message_handler = tokio::spawn(Self::message_task(
            config.clone(),
            message_rx,
            content_tx.clone(),
        ));
        
        // 启动渲染任务
        let render_handler = tokio::spawn(Self::render_task(
            config,
            content_rx,
        ));

        Self { 
            message_tx,
            content_tx,
            message_handler:Some(message_handler),
            render_handler:Some(render_handler),
        }
    }

    /// 推送内容到渲染队列
    pub async fn push_content(&self, content: &str) -> Result<(), tokio::sync::mpsc::error::SendError<RenderMessage>> {
        self.message_tx.send(RenderMessage::Content(content.to_string())).await
    }

    /// 设置状态
    pub async fn set_status(&self, status: ResponseStatus) -> Result<(), tokio::sync::mpsc::error::SendError<RenderMessage>> {
        self.message_tx.send(RenderMessage::SetStatus(status)).await
    }

    /// 消息处理任务 - 负责接收和处理所有输入消息
    async fn message_task(
        config: RenderConfig,
        mut message_rx: Receiver<RenderMessage>,
        content_tx: Sender<String>,
    ) {
        let mut shared_state = RenderState::new();
        
        while let Some(message) = message_rx.recv().await {
            match message {
                RenderMessage::Content(content) => {
                    // 将内容发送到渲染任务
                    if content_tx.send(content).await.is_err() {
                        break;
                    }
                },
                RenderMessage::SetStatus(new_status) => {
                    shared_state.status = new_status;
                    if new_status == ResponseStatus::Done || new_status == ResponseStatus::Error {
                        // 发送一个空字符串来触发渲染任务的状态更新
                        let _ = content_tx.send(String::new()).await;
                    }
                },
                RenderMessage::Stop => {
                    let _ = content_tx.send(String::new()).await;
                    break;
                },
            }
        }
    }

    /// 渲染任务 - 负责以固定速度渲染内容
    async fn render_task(config: RenderConfig, mut content_rx: Receiver<String>) {
        let mut stdout = stdout();
        let mut state = RenderState::new();

        // 初始化显示
        if !config.pure {
            let _ = stdout.queue(cursor::MoveToNextLine(3));
            let _ = stdout.queue(cursor::SavePosition);
            Self::render_status_bar(&mut stdout, &state, &config);
            let _ = stdout.write_all(format!("\n🤖 {}: ", config.model_name).as_bytes());
            let _ = stdout.flush();
        }

        let char_interval = Duration::from_secs_f32(1.0 / config.type_speed as f32);
        let mut last_update = Instant::now();
        
        loop {
            // 检查是否有新内容
            if let Ok(content) = content_rx.try_recv() {
                if !content.is_empty() {
                    state.content_queue.push_back(content);
                }
            }

            // 处理字符渲染
            if state.current_position >= state.current_content.len() {
                if let Some(next_content) = state.content_queue.pop_front() {
                    state.current_content = next_content;
                    state.current_position = 0;
                }
            }

            // 渲染字符
            if state.current_position < state.current_content.len() && state.last_char_time.elapsed() >= char_interval {
                let ch = state.current_content[state.current_position..].chars().next().unwrap();
                let _ = stdout.write_all(ch.to_string().as_bytes());
                let _ = stdout.write_all(cursor::position().unwrap().1.to_string().as_bytes());
                let _ = stdout.flush();
                state.current_position += ch.len_utf8();
                state.last_char_time = Instant::now();
            }

    

            // 如果没有内容要处理，短暂休眠
            if !state.has_more_content() {
                sleep(Duration::from_millis(10)).await;  // 缩短休眠时间以提高响应性
            }

            // 如果当前内容打印完了，获取新内容
            if state.current_position >= state.current_content.len() {
                if let Some(next_content) = state.content_queue.pop_front() {
                    state.current_content = next_content;
                    state.current_position = 0;
                }
            }

            // 打印字符
            if state.current_position < state.current_content.len() {
                if state.last_char_time.elapsed() >= char_interval {
                    let ch = state.current_content[state.current_position..].chars().next().unwrap();
                    let _ = stdout.write_all(ch.to_string().as_bytes());
                    let _ = stdout.flush();
                    state.current_position += ch.len_utf8();
                    state.last_char_time = Instant::now();
                }
            }

            // 更新状态栏
            if !config.pure && state.last_status_update.elapsed() >= Duration::from_millis(config.status_refresh_interval) {
                let _ = stdout.queue(cursor::SavePosition);
                Self::render_status_bar(&mut stdout, &state, &config);
                // let _ = stdout.queue(cursor::RestorePosition);
                let _ = stdout.flush();
                let _ = stdout.queue(cursor::RestorePosition);
                
                

                state.last_status_update = Instant::now();
            }

            // 如果没有更多内容要处理，短暂休眠
            if !state.has_more_content() {
                sleep(Duration::from_millis(50)).await;
            }
        }
    }

    /// 渲染状态栏
    fn render_status_bar(
        stdout: &mut std::io::Stdout,
        state: &RenderState,
        config: &RenderConfig,
    ) {
        let elapsed = state.start_time.elapsed();
        let elapsed_secs = elapsed.as_secs();
        let elapsed_str = format!("{:02}:{:02}", elapsed_secs / 60, elapsed_secs % 60);
        
        let _ = stdout.queue(cursor::MoveToColumn(0));
        let _ = stdout.queue(cursor::MoveToRow(state.start_row));
        let _ = stdout.queue(terminal::Clear(terminal::ClearType::CurrentLine));
        let _ = stdout.queue(SetBackgroundColor(Color::DarkBlue));
        let _ = stdout.write_all(
            format!(
                "{} {} | Model: {} | Prompt: {}",
                state.status.as_emoji(),
                elapsed_str,
                config.model_name,
                config.prompt_name,
            )
            .as_bytes(),
        );
        let _ = stdout.queue(SetBackgroundColor(Color::Reset));
    }

    pub async fn wait(&mut self){
        let _=self.message_handler.take().unwrap().await;
        let _=self.render_handler.take().unwrap().await;
    }
}


impl Drop for ResponseRenderer {
    fn drop(&mut self) {
        // 创建一个新的运行时来发送停止信号
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let _ = self.message_tx.send(RenderMessage::Stop).await;
        });
    }
}
