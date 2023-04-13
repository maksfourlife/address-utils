```
Usage: address-util [OPTIONS] --target <TARGET> --mask <MASK> <COMMAND>

Commands:
  address           Generate EOA account address
  contract-address  Generate address for contract deployed with nonce
  create2-address   Generate salt for create-2 deployed contract
  help              Print this message or the help of the given subcommand(s)

Options:
      --target <TARGET>    
      --mask <MASK>        
      --n-cores <N_CORES>  Number of cores to use [default: 31]
      --n-iter <N_ITER>    Debug info every n iterations [default: 100000]
  -h, --help               Print help
```