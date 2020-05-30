module github.com/BigRedEye/dc-hw/gateway

go 1.14

replace github.com/BigRedEye/dc-hw/api/proto => ../api/proto

require (
	github.com/BigRedEye/dc-hw/api/proto v0.0.0-00010101000000-000000000000
	github.com/grpc-ecosystem/grpc-gateway v1.14.5
	github.com/sirupsen/logrus v1.6.0
	github.com/spf13/viper v1.7.0
	google.golang.org/grpc v1.29.1
)
