package config

import (
	log "github.com/sirupsen/logrus"

	"github.com/spf13/viper"
)

type Config struct {
	ShopAddress string `mapstructure:"shop_address"`
	AuthAddress string `mapstructure:"auth_address"`
	BindAddress string `mapstructure:"bind_address"`

	LogFile string `mapstructure:"log_file"`
}

func LoadConfig() (*Config, error) {
	viper.SetConfigName("gw")
	viper.SetEnvPrefix("gw")
	viper.AddConfigPath(".")

	viper.BindEnv("BIND_ADDRESS")
	viper.BindEnv("AUTH_ADDRESS")
	viper.BindEnv("SHOP_ADDRESS")
	viper.BindEnv("LOG_FILE")

	viper.SetDefault("shop_address", ":7780")
	viper.SetDefault("auth_address", ":7781")
	viper.SetDefault("bind_address", ":7782")

	if err := viper.ReadInConfig(); err != nil {
		if _, ok := err.(viper.ConfigFileNotFoundError); ok {
			log.Warn("Config file not found")
		} else {
			return nil, err
		}
	}

	var config Config
	if err := viper.Unmarshal(&config); err != nil {
		return nil, err
	}

	return &config, nil
}
