# aidoc 使用指南

## explain

```bash
cargo run -p aidoc-cli -- explain
```

说明：打印工具行为边界与 pack 合并协议。

## paste

```bash
printf "bash: pip: command not found\n" | cargo run -p aidoc-cli -- paste --format md
```

说明：输入原始日志，输出结构化诊断包。

## diagnose

```bash
cargo run -p aidoc-cli -- diagnose --format md
```

说明：执行只读探针，采集环境快照。

## pack

```bash
{
  printf "bash: pip: command not found\n"
  printf "\n---AIDOC-SECTION-BREAK---\n"
  printf '{"os":"linux","arch":"x86_64","shell":"bash","elevated":false,"path_preview":[],"toolchains":[],"proxy":{"http_proxy":null,"https_proxy":null,"no_proxy":null},"network":[]}\n'
} | cargo run -p aidoc-cli -- pack --format md
```

说明：合并日志与快照，生成最终 handover pack。
