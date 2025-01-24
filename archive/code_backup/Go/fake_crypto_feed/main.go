package main

import (
    "context"
    "log"
    "os"
    "os/signal"
    "sync"
    "syscall"

    "github.com/mag1cfrog/stock_trading_bot/code/Go/fake_crypto_feed/client"
    "github.com/mag1cfrog/stock_trading_bot/code/Go/fake_crypto_feed/sender"
)

func main() {
    ctx, cancel := context.WithCancel(context.Background())
    defer cancel()

    sigs := make(chan os.Signal, 1)
    signal.Notify(sigs, syscall.SIGINT, syscall.SIGTERM)

    var wg sync.WaitGroup

    wg.Add(1)
    go func() {
        defer wg.Done()
        if err := sender.StartServer(ctx, &wg); err != nil {
            log.Println("Sender error:", err)
        }
    }()

    wg.Add(1)
    go func() {
        defer wg.Done()
        if err := client.StartClient(ctx); err != nil {
            log.Println("Client error:", err)
        }
    }()

    <-sigs
    cancel()
    wg.Wait()
    log.Println("Shutting down")
}