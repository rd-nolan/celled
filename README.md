# Celled

本地桌面 Excel 转换工具（仓库 / 包名仍为 `celld`）。上传一份模板，确认表头后，把多份源 Excel 映射到模板字段，再按模板列顺序导出 `.xlsx`。文件与字段都在本机处理，不上传网络。

技术栈：Tauri 2 + Vue 3 + Rust。

## 用法

界面分三步。未确认模板前，后两步不可进入。

### 1. 上传模板

1. 拖拽或点击选择模板（`.xlsx` / `.xls` / `.xlsm`）。
2. 核对 Sheet、表头所在行，并查看预览。
3. 点击 **确认模板**。确认后才会计算字段 Embedding，并允许导入数据。

### 2. 上传源数据文件

1. 拖拽或点击 **添加文件**，可一次加入多份源 Excel。已添加的文件列在右侧。
2. 逐个文件核对 Sheet、表头行，并确认字段映射。系统会推荐映射，来源包括：
   - **精确匹配**（Exact）
   - **历史匹配**（History）
   - **别名匹配**（Alias）
   - **AI 推荐**（Embedding）
3. 可改选模板字段，或选择「不映射 / 忽略此列」。AI 只推荐，不会自动提交。
4. 每个文件点 **确认当前文件映射**。全部确认后点 **下一步**。

### 3. 汇总数据

1. 核对已确认的文件列表，点 **开始转换**，选择输出目录。
2. 成功会显示「转换完成」和输出路径；失败会显示错误信息。
3. 每个源文件生成一份 `{原文件名}_converted.xlsx`，列顺序与模板一致。需要时可 **再次转换**。

## 开发运行

仓库使用 `pnpm-lock.yaml`，也可用 npm。

```bash
pnpm install   # 或 npm install
npm run tauri dev
```

未提供 `model.onnx` 时，Embedding 会回退到本地 mock，应用仍可运行。

## Windows 安装

请安装 CI 产物里的 **`Celled_*_x64-setup.exe`（NSIS）或 `Celled_*_x64_*.msi`**，不要单独拷贝 `celld.exe`。

当前默认构建不含 ONNX Runtime（仓库里没有 `model.onnx`）。安装后即可打开应用，界面里 Embedding 后端为 `mock`。旧安装包把 DirectML / MSVCP140 写进了 exe 的导入表却没随安装包分发，会在启动时报 `0xc000007b`。需要重新打一次 `tauri build` / CI 包。
