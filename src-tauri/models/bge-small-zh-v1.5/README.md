# bge-small-zh-v1.5

这是面向中文语义检索的轻量级 embedding 模型，适合 RAG、文档搜索和文本相似度计算。

当前目录使用的是 ONNX INT8 量化版本，适合通过 Rust 驱动，并部署到桌面电脑、Android 或 iOS 等设备。

## 模型信息

- 基础模型：`BAAI/bge-small-zh-v1.5`
- ONNX 来源：`Xenova/bge-small-zh-v1.5`
- 模型文件：`onnx/model_quantized.onnx`
- 模型大小：约 23 MB
- 向量维度：512
- 最大输入长度：512 tokens
- 适用语言：中文为主，也适合中英文混合技术文档
- 适用内容：Linux、Kubernetes、Docker、Python、Shell、配置文件和技术说明

## 文件说明

| 文件 | 用途 |
| --- | --- |
| `onnx/model_quantized.onnx` | ONNX INT8 量化模型 |
| `tokenizer.json` | Hugging Face Tokenizer 配置和词表 |
| `tokenizer_config.json` | Tokenizer 参数 |
| `special_tokens_map.json` | 特殊 token 配置 |
| `vocab.txt` | 词表文件 |
| `config.json` | Transformer 模型配置 |

## Rust 集成

推荐使用以下 crate：

```toml
[dependencies]
ort = "2"
tokenizers = "0.21"
ndarray = "0.16"
```

模型路径：

```text
models/bge-small-zh-v1.5/onnx/model_quantized.onnx
```

Tokenizer 路径：

```text
models/bge-small-zh-v1.5/tokenizer.json
```

基本推理流程如下：

1. 使用 `tokenizers` 加载 `tokenizer.json`。
2. 对文本进行 tokenize，得到 `input_ids` 和 `attention_mask`。
3. 将两个输入传给 ONNX 模型。
4. 从模型输出中取得 token embeddings。
5. 使用 `attention_mask` 做 mean pooling，得到句向量。
6. 对句向量做 L2 normalization。
7. 使用 cosine similarity 进行检索或相似度比较。

## Pooling 和归一化

不能直接把某一个 token 的输出当作最终向量。应当根据 `attention_mask` 忽略 padding token，再做 mean pooling：

```text
pooled = sum(token_embedding[i] * attention_mask[i])
         / sum(attention_mask[i])
```

然后做 L2 normalization：

```text
normalized = pooled / sqrt(sum(pooled[i] * pooled[i]))
```

两个向量都完成归一化后，可以直接使用点积作为 cosine similarity。

## RAG 使用建议

- 中文技术文档建议按标题、段落和代码块切分。
- 普通文本 chunk 建议控制在约 300～600 个中文字符。
- 相邻 chunk 可以保留约 50～100 个字符的重叠。
- Linux、K8s、Docker、`kubectl` 等技术名词应原样保留，不要翻译。
- 代码块尽量作为独立内容保存，避免和过多自然语言混在一起。
- 查询文本和文档文本应使用相同的 tokenizer、pooling 和归一化流程。

## 完整性校验

`onnx/model_quantized.onnx` 的 SHA-256：

```text
15b717c382bcb518ba457b93ea6850ede7f4f1cd8937454aa06972366cd19bcc
```
