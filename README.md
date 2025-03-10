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

## Star History

<a href="https://www.star-history.com/#kleedodo/aicheck&Date">
 <picture>
   <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/svg?repos=kleedodo/aicheck&type=Date&theme=dark" />
   <source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/svg?repos=kleedodo/aicheck&type=Date" />
   <img alt="Star History Chart" src="https://api.star-history.com/svg?repos=kleedodo/aicheck&type=Date" />
 </picture>
</a>