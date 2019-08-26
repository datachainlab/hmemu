module github.com/bluele/hmemu

go 1.12

replace (
	github.com/go-interpreter/wagon v0.0.0 => github.com/perlin-network/wagon v0.3.1-0.20180825141017-f8cb99b55a39
	github.com/tendermint/tendermint v0.32.2-rc2 => github.com/bluele/tendermint v0.32.2-rc2
)

require (
	github.com/beorn7/perks v1.0.0 // indirect
	github.com/bluele/hypermint v0.0.0-20190826154745-87c53298d313
	github.com/ethereum/go-ethereum v1.8.21
	github.com/go-logfmt/logfmt v0.4.0 // indirect
	github.com/gorilla/websocket v1.4.0 // indirect
	github.com/pelletier/go-toml v1.3.0 // indirect
	github.com/perlin-network/life v0.0.0-20190402092845-c30697b41680 // indirect
	github.com/prometheus/client_golang v0.9.2 // indirect
	github.com/prometheus/client_model v0.0.0-20190129233127-fd36f4220a90 // indirect
	github.com/prometheus/common v0.3.0 // indirect
	github.com/prometheus/procfs v0.0.0-20190416084830-8368d24ba045 // indirect
	github.com/spf13/afero v1.2.2 // indirect
	github.com/spf13/cobra v0.0.3 // indirect
	github.com/spf13/jwalterweatherman v1.1.0 // indirect
	github.com/spf13/viper v1.3.2 // indirect
	github.com/tendermint/tendermint v0.32.2-rc2
	github.com/tendermint/tm-db v0.1.1
	google.golang.org/appengine v1.5.0 // indirect
)
