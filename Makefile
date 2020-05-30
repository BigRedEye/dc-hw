.PHONY: gateway auth shop

gateway_protos:
	protoc -I. -I./third-party/grpc-gateway/third_party/googleapis --go_out=plugins=grpc,paths=source_relative:. api/proto/*.proto
	protoc -I. -I./third-party/grpc-gateway/third_party/googleapis --grpc-gateway_out=logtostderr=true,paths=source_relative:. api/proto/*.proto

gateway: gateway_protos
	cd gateway; go build .

auth:
	cargo build --release --bin auth

shop:
	cargo build --release --bin shop
