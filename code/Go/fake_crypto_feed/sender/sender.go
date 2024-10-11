package sender

import (
	"context"
	"encoding/json"
	"log"
	"math/rand"
	"net/http"
	"os"
	"time"

	"github.com/gorilla/websocket"
	"github.com/prometheus/client_golang/prometheus"
	"github.com/prometheus/client_golang/prometheus/promhttp"
	"github.com/shirou/gopsutil/process"
)

var (
	bytesSent = prometheus.NewCounter(prometheus.CounterOpts{
		Name: "Go_sender_bytes_sent_total",
		Help: "Total bytes sent over the network",
	})
	messagesSent = prometheus.NewCounter(prometheus.CounterOpts{
		Name: "Go_sender_messages_sent_total",
		Help: "Total number of messages sent",
	})
	errorsEncountered = prometheus.NewCounter(prometheus.CounterOpts{
		Name: "Go_sender_errors_total",
		Help: "Total number of errors encountered",
	})
	cpuUsage = prometheus.NewGauge(prometheus.GaugeOpts{
		Name: "Go_sender_cpu_usage_percent",
		Help: "CPU usage percentage",
	})
	ramUsage = prometheus.NewGauge(prometheus.GaugeOpts{
		Name: "Go_sender_ram_usage_mb",
		Help: "RAM usage in MB",
	})
)

func init() {
	prometheus.MustRegister(bytesSent, messagesSent, errorsEncountered, cpuUsage, ramUsage)
}

var upgrader = websocket.Upgrader{}

type PriceData struct {
	Bid float64 `json:"bid"`
	Ask float64 `json:"ask"`
}

func StartServer(ctx context.Context) error {
	go func() {
		http.Handle("/metrics", promhttp.Handler())
		if err := http.ListenAndServe(":9000", nil); err != nil {
			log.Println("Metrics server error:", err)
		}
	}()

	go updateUsageMetrics(ctx)

	http.HandleFunc("/ws", handleConnections)

	srv := &http.Server{Addr: ":8081"}

	go func() {
		<-ctx.Done()
		srv.Close()
	}()

	log.Println("WebSocket server started on :8081")
	if err := srv.ListenAndServe(); err != http.ErrServerClosed {
		return err
	}
	return nil
}

func handleConnections(w http.ResponseWriter, r *http.Request) {
	conn, err := upgrader.Upgrade(w, r, nil)
	if err != nil {
		log.Println("Upgrade error:", err)
		errorsEncountered.Inc()
		return
	}
	defer conn.Close()

	for {
		select {
		case <-r.Context().Done():
			return
		default:
			data := generateFakeData()
			message, err := json.Marshal(data)
			if err != nil {
				log.Println("JSON marshal error:", err)
				errorsEncountered.Inc()
				continue
			}
			err = conn.WriteMessage(websocket.TextMessage, message)
			if err != nil {
				log.Println("Write message error:", err)
				errorsEncountered.Inc()
				return
			}
			bytesSent.Add(float64(len(message)))
			messagesSent.Inc()
			time.Sleep(1 * time.Second)
		}
	}
}

func generateFakeData() PriceData {
	bid := rand.Float64()*1000 + 30000
	ask := bid + rand.Float64()*100
	return PriceData{Bid: bid, Ask: ask}
}

func updateUsageMetrics(ctx context.Context) {
	proc, err := process.NewProcess(int32(os.Getpid()))
	if err != nil {
		log.Println("Process error:", err)
		return
	}
	ticker := time.NewTicker(2 * time.Second) // update every 2 seconds
	defer ticker.Stop() // prevent memory leak

	for {
		select {
		case <-ctx.Done():
			return  // gracefully exit
		case <-ticker.C:
			cpu, err := proc.CPUPercent()
			if err == nil {
				cpuUsage.Set(cpu)
			}
			memInfo, err := proc.MemoryInfo()
			if err == nil {
				ramUsage.Set(float64(memInfo.RSS) / 1024 / 1024)
			}
		}
	}
}
