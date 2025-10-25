# MCBE Text Impact

一个用于 Minecraft Bedrock Edition (MCBE) 的文本渲染工具，可以将带有格式的文本渲染成精确对齐的图像。

## 功能特点

- 将 MCBE `tellraw` 命令中的 JSON 文本或普通带格式文本渲染成 PNG 图片
- 支持 Minecraft 格式代码（如 §c 红色文字、§l 粗体等）
- 精确的字体宽度计算，确保像素级对齐
- 提供 Web 界面进行实时预览
- 支持多种对齐方式和内边距设置

## 使用方法

```bash
# 启动 Web 服务
cargo run

# 在浏览器中打开 http://localhost:8080
```

## 许可证

MIT