# 图像处理与降噪工具

这是一个使用 Rust 开发的图像处理工具，提供了多种图像降噪算法和图像增强功能。该工具具有图形用户界面，支持实时预览处理效果。

## 功能特点

- 支持多种图像格式：
  - JPG/JPEG
  - PNG
  - GIF

- 支持多种降噪算法：
  - 均值滤波 (Mean Filter)
  - 高斯滤波 (Gaussian Filter)
  - 中值滤波 (Median Filter)
  - 双边滤波 (Bilateral Filter)
  - 非局部均值滤波 (Non-Local Means)
  - 全变分降噪 (Total Variation)

- 图像增强功能：
  - 亮度调整
  - 对比度调整
  - 锐化处理

- 高级特性：
  - 并行处理支持
  - 自动优化功能
  - 实时预览
  - 处理时间统计
  - 图像导出功能

## 系统要求

- Rust 1.70.0 或更高版本
- Windows 操作系统

## 安装

1. 确保已安装 Rust 开发环境
2. 创建新的 Rust 项目：
   ```bash
   cargo new image_denoise
   cd image_denoise
   ```
3. 替换项目文件：
   - 将 `src/main.rs` 替换为项目中的 `src/main.rs`
   - 将 `Cargo.toml` 替换为项目中的 `Cargo.toml`
4. 编译项目：
   ```bash
   cargo build --release
   ```

## 使用方法

1. 运行程序：
   ```bash
   cargo run --release
   ```

2. 在图形界面中：
   - 点击 "Select Image" 选择要处理的图片
   - 选择降噪算法和参数
   - 调整图像增强参数
   - 点击 "Apply Denoising" 应用处理
   - 使用 "Auto Optimize" 进行自动优化
   - 点击 "Export Image" 保存处理后的图片

## 并行处理

程序支持并行处理以提高性能：
- 启用 "Use Parallel Processing" 选项
- 调整 Block Size 参数（32-256像素）以优化性能

## 依赖项

- eframe: 用于构建图形界面
- image: 图像处理库
- rfd: 文件对话框
- rayon: 并行计算支持
- winapi: Windows API 接口

## 贡献

欢迎提交 Issue 和 Pull Request 来帮助改进这个项目。 
