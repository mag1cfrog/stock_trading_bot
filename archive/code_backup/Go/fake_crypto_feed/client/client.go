package client

import (
    "context"
    "log"
    "time"

    "github.com/gorilla/websocket"
)

func StartClient(ctx context.Context) error {
    url := "ws://localhost:8081/ws"
    var conn *websocket.Conn
    var err error
    backoff := 1 * time.Second

    for {
        select {
        case <-ctx.Done():
            if conn != nil {
                conn.Close()
            }
            return nil
        default:
            if conn == nil {
                conn, _, err = websocket.DefaultDialer.Dial(url, nil)
                if err != nil {
                    log.Println("Connection error:", err)
                    time.Sleep(backoff)
                    if backoff < 30*time.Second {
                        backoff *= 2
                    }
                    continue
                }
                backoff = 1 * time.Second
            }

            _, message, err := conn.ReadMessage()
            if err != nil {
                log.Println("Read error:", err)
                conn.Close()
                conn = nil
                continue
            }
            log.Printf("Received: %s", message)
        }
    }
}