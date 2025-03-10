# AI Check 

本工具可以获取 deepseek 和 siliconflow的余额

也可以测试gemini key 是否可用，但**不建议一次性测太多，有可能会触发google的风控**

```sh
Usage: aicheck [OPTIONS] --type <TYPE> <KEYS_FILE>

Arguments:
  <KEYS_FILE>  

Options:
  -t, --type <TYPE>  [possible values: siliconflow, deepseek, gemini]
  -n, --num <NUM>    
  -h, --help         Print help
  -V, --version      Print version
```

其中的-n 参数是一秒内发起的请求数量

key每行一个

