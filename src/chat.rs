use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use async_openai::{
    config::OpenAIConfig,
    types::chat::{
        ChatCompletionRequestSystemMessage,
        ChatCompletionRequestUserMessage,
        CreateChatCompletionRequestArgs,
    },
    Client,
};
use chrono::Local;
use once_cell::sync::OnceCell;

// -------------------- 全局共享状态 --------------------
static CLIENT: OnceCell<Client<OpenAIConfig>> = OnceCell::new();
static ANON_SETTING: OnceCell<String> = OnceCell::new();
static DATA_PATH: OnceCell<PathBuf> = OnceCell::new();

// -------------------- 初始化函数 --------------------
/// 在机器人启动时调用一次。
/// 参数 `data_path` 应该来自 `RuntimeBot::get_data_path()`。
pub async fn init_anon_bot(data_path: &Path) -> Result<(), Box<dyn Error>> {
    // 1. 保存数据根目录
    DATA_PATH.set(data_path.to_path_buf()).map_err(|_| "DATA_PATH already set")?;

    // 2. 读取 API Key 和 Anon 设定（文件必须存在于 data_path 下）
    let api_key_path = data_path.join("api_key.txt");
    let anon_path = data_path.join("anon.txt");

    let api_key = fs::read_to_string(&api_key_path)
        .map_err(|e| format!("读取 api_key.txt 失败: {}", e))?;
    let anon_setting = fs::read_to_string(&anon_path)
        .map_err(|e| format!("读取 anon.txt 失败: {}", e))?;

    // 3. 创建 Client
    unsafe {
        std::env::set_var("OPENAI_BASE_URL", "https://api.deepseek.com");
        std::env::set_var("OPENAI_API_KEY", api_key);
    }
    let client = Client::new();

    // 4. 存入全局
    CLIENT.set(client).map_err(|_| "CLIENT already set")?;
    ANON_SETTING.set(anon_setting).map_err(|_| "ANON_SETTING already set")?;

    // 确保 chat_files 目录存在（用于存放对话记录）
    let chat_files_dir = data_path.join("chat_files");
    fs::create_dir_all(&chat_files_dir)?;

    Ok(())
}

// -------------------- 聊天函数 --------------------
/// 输入用户消息，返回爱音的回复。
/// 同时自动保存 request/response JSON 到 `data_path/chat_files/时间戳/` 下。
pub async fn get_anon_reply(user_input: &str) -> Result<String, Box<dyn Error>> {
    // 获取全局状态（如果未初始化则报错）
    let client = CLIENT.get().ok_or("机器人未初始化，请先调用 init_anon_bot")?;
    let anon_setting = ANON_SETTING.get().ok_or("未找到爱音角色设定")?;
    let data_path = DATA_PATH.get().ok_or("未找到数据目录")?;

    // 1. 构建请求
    let request = CreateChatCompletionRequestArgs::default()
        .max_tokens(512u32)
        .model("deepseek-v4-flash")
        .messages([
            ChatCompletionRequestSystemMessage::from(anon_setting.clone()).into(),
            ChatCompletionRequestUserMessage::from(user_input.to_string()).into(),
        ])
        .build()?;

    // 2. 创建本次对话的存档文件夹（按时间戳）
    let timestamp = Local::now().format("%Y-%m-%d-%H-%M-%S").to_string();
    let chat_session_dir = data_path
        .join("chat_files")
        .join(&timestamp);
    fs::create_dir_all(&chat_session_dir)?;

    // 3. 保存请求 JSON
    let request_path = chat_session_dir.join("request.json");
    fs::write(&request_path, serde_json::to_string_pretty(&request)?)?;
    // 你可以选择是否打印保存路径，此处静默保存

    // 4. 发送请求
    let response = client.chat().create(request).await?;

    // 5. 保存完整响应 JSON
    let response_path = chat_session_dir.join("response.json");
    fs::write(&response_path, serde_json::to_string_pretty(&response)?)?;

    // 6. 提取回复内容
    let reply_text = response.choices
        .into_iter()
        .find_map(|choice| choice.message.content)
        .unwrap_or_else(|| "（无文本内容）".to_string());

    Ok(reply_text)
}