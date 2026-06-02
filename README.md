# anon-chat

与千早爱音聊天的 [Kovi](https://kovi.thricecola.com/) 插件

使用前需要在 `data/anon-chat` 中放好两个文件 `api_key.txt` 和 `anon.txt` ，分别存储大语言模型的 api key 和千早爱音 prompt. 

如果使用的大语言模型不是 deepseek-v4-flash，请自行修改 `src/chat.rs` 中的相关配置。
