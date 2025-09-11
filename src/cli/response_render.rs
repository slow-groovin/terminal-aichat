use crossterm::{
    cursor, style::{Print, ResetColor, SetBackgroundColor, Stylize}, QueueableCommand
};
use std::{
    io::{self, Write, stdout},
    time::{Duration, Instant},
};
use tokio::{
    sync::mpsc::{self, Receiver, Sender},
    time::sleep,
};

use crate::{log_debug, log_trace};

/// 渲染配置
#[derive(Clone)]
pub struct RenderConfig {
    pub pure: bool,
    pub model_config_name: String,
    pub model_name: String,
    pub prompt_config_name: String,
    /// 打字机效果的速度（字符/秒）
    pub type_speed: u32,
}

/// 响应渲染器
pub struct ResponseRenderer {
    /// 开始时间
    start_time: Instant,
}

impl ResponseRenderer {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
        }
    }

    pub fn start_render(
        &self,
        config: RenderConfig,
    ) -> (Sender<String>, tokio::task::JoinHandle<()>) {
        let (message_tx, message_rx) = mpsc::channel(100);
        let start_time = self.start_time; // Copy start_time

        // 仅启动一个渲染任务，处理所有消息
        let render_handler = tokio::spawn(async move {
            Self::render_task_impl(start_time, config, message_rx).await;
        });
        return (message_tx, render_handler);
    }

    /// 渲染任务 - 处理所有消息并渲染
    async fn render_task_impl(
        start_time: Instant,
        config: RenderConfig,
        mut message_rx: Receiver<String>,
    ) {
        let mut stdout = stdout();
        if !config.pure {
            let _ = Self::render_status_bar(&mut stdout, &config);
            log_trace!("Render Status Bar.");
        }

        let char_interval = Duration::from_secs_f32(1.0 / config.type_speed as f32);

        // 非阻塞处理所有待处理消息
        while let Some(value) = message_rx.recv().await {
            //渲染字符
            Self::print_with_interval(value.as_str(), char_interval).await;
        }

        let cost = Instant::now() - start_time;

        log_debug!("Message Receiver Exit.");

        if !config.pure {
            // 打印尾部
            let _ = stdout.queue(Print(format!("✅{:#?}", cost).dark_green()));
        }

        // 结束时换行
        let _ = stdout.queue(Print("\n"));
        let _ = stdout.flush();
    }

    /// 渲染状态栏（固定在status_row）
    fn render_status_bar(
        stdout: &mut std::io::Stdout,
        config: &RenderConfig,
    ) -> Result<(), io::Error> {
        let _ = stdout
            .queue(SetBackgroundColor(crossterm::style::Color::DarkBlue))?
            .queue(Print("model:"))?
            .queue(Print(config.model_config_name.clone().dark_yellow().bold()))?
            .queue(Print("   prompt:".on_dark_blue()))?
            .queue(Print(config.prompt_config_name.clone().on_dark_blue().bold()))?
            .queue(Print("     🤖".on_dark_blue()))?
            .queue(Print(config.model_name.clone().cyan().bold().on_dark_blue()))?
            .queue(ResetColor)?
            .queue(Print("\n"))?
            .flush()?;

        stdout
            .queue(cursor::MoveToNextLine(1))?
            .queue(cursor::EnableBlinking)?
            .flush()
    }

    /// 异步函数：按给定时间间隔打印字符串的每个字符
    async fn print_with_interval(s: &str, char_interval: Duration) {
        for c in s.chars() {
            print!("{}", c);
            // 立即刷新 stdout，否则可能会缓冲不立即显示
            use std::io::Write;
            std::io::stdout().flush().unwrap();

            sleep(char_interval).await;
        }
    }
}
