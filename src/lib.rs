use kovi::PluginBuilder as plugin;

mod chat;

#[kovi::plugin]
async fn main() {
    let bot = plugin::get_runtime_bot();
    let data_path = bot.get_data_path();   // 例如 ./data/my_anon_plugin/

    // 启动时初始化一次
    if let Err(e) = chat::init_anon_bot(&data_path).await {
        eprintln!("初始化爱音机器人失败: {}", e);
        return;
    }

    // 注册消息处理命令，例如 /chat 或直接匹配关键词
    plugin::on_group_msg(|msg| async move {
        let text = msg.get_text();
        if text.starts_with("爱音爱音 ") {
            let user_input = text.trim_start_matches("爱音爱音 ").trim();
            match chat::get_anon_reply(user_input).await {
                Ok(reply) => msg.reply(reply),
                Err(e) => msg.reply(format!("出错了: {}", e)),
            }
        }
    });
}
