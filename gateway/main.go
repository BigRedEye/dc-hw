package main

import (
	"context"
	"fmt"
	"io"
	"net/http"
	"os"
	"path"
	"runtime"

	gw "github.com/grpc-ecosystem/grpc-gateway/runtime"
	log "github.com/sirupsen/logrus"
	"google.golang.org/grpc"

	pb "github.com/BigRedEye/dc-hw/api/proto"
	config "github.com/BigRedEye/dc-hw/gateway/config"
)

func initLogging(conf *config.Config) {
	log.SetFormatter(&log.TextFormatter{
		FullTimestamp: true,
		CallerPrettyfier: func(f *runtime.Frame) (string, string) {
			filename := path.Base(f.File)
			return "", fmt.Sprintf("%s:%d", filename, f.Line)
		},
	})
	log.SetReportCaller(true)

	if conf.LogFile != "" {
		log.WithField("file", conf.LogFile).Info("Initializing log file")
		logFile, err := os.OpenFile(conf.LogFile, os.O_APPEND|os.O_CREATE|os.O_WRONLY, 0o644)
		if err != nil {
			log.WithError(err).Fatalln("Failed to open log file")
		}
		mw := io.MultiWriter(os.Stderr, logFile)
		log.SetOutput(mw)
	}
}

func runGrpcGateway(conf *config.Config) {
	ctx := context.Background()
	ctx, cancel := context.WithCancel(ctx)
	defer cancel()

	mux := gw.NewServeMux()
	opts := []grpc.DialOption{grpc.WithInsecure()}
	err := pb.RegisterAuthHandlerFromEndpoint(ctx, mux, conf.AuthAddress, opts)
	if err != nil {
		log.WithError(err).Fatalln("Failed to start grpc-gateway")
	}
	err = pb.RegisterShopHandlerFromEndpoint(ctx, mux, conf.ShopAddress, opts)
	if err != nil {
		log.WithError(err).Fatalln("Failed to start grpc-gateway")
	}

	log.Println("Starting grpc-gateway server at", conf.BindAddress)
	err = http.ListenAndServe(conf.BindAddress, mux)
	log.WithError(err).Fatalln("Grpc-gateway server failed")
}

func main() {
	conf, err := config.LoadConfig()
	if err != nil {
		log.WithError(err).Fatalln("Failed to load config")
	}

	initLogging(conf)
	runGrpcGateway(conf)
}
