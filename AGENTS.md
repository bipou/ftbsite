## Git 提交信息规范

### 语言
中文优先、英文辅助

### 格式
<类型>: <简洁描述>

## 编码规范

### 规则
1. 严格依据最新资料，遵循 Leptos 规范和实践，细粒度、信号、响应式，及同构模式
2. view! 中用 t!, 无法用 t! 时备选 t_display, 用 td_string 须特别说明，禁 t_string
3. view! 分支或栈溢：Either 是首选；拆分为独立组件是根本；标准库也好，禁 into_any
4. 宏/match 的组合，或 match 优先于 if/else，禁无谓冗余克隆 clone，能避免 format 则避免
5. 样式须级联，禁并列组件重复样式
6. 遵循 SurrealDB 规范和实践
7. 代码注释中文优先，英文辅助
