package sender

import (
	"context"
	"encoding/json"
	"log"
	"math"
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
	Timestamp string `json:"timestamp"`
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

	// Create a data generator
	dataGenerator := generateFakeDataGenerator()

	for {
		select {
		case <-r.Context().Done():
			return
		default:
			// Get data and interval
            data, interval := dataGenerator()

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

			// Sleep for the calculated interval
            time.Sleep(interval)
		}
	}
}

// Function to generate a data generator maintaining state
func generateFakeDataGenerator() func() (PriceData, time.Duration) {
    // Initialize bidPrice and askPrice
    bidPrice := rand.Float64()*10000 + 10000 // Between 10000 and 20000
    askPrice := bidPrice + rand.Float64()*10 + 5 // Bid + 5 to 15

    return func() (PriceData, time.Duration) {
        // Calculate jitter between -0.1 and 0.1
        jitter := rand.Float64()*0.2 - 0.1
        interval := time.Duration(200*(1+jitter)) * time.Millisecond

        // Choose direction -1 or 1
        directions := []float64{-1, 1}
        direction := directions[rand.Intn(len(directions))]

        // Change percentage between 0.01 and 0.03
        changePercentage := rand.Float64()*0.02 + 0.01

        // Calculate bid and ask changes
        bidChange := bidPrice * changePercentage * direction
        askChange := askPrice * (changePercentage + rand.Float64()*0.01 - 0.005) * direction

        // Update bidPrice and askPrice
        bidPrice += bidChange
        askPrice += askChange

        // Ensure askPrice > bidPrice
        if askPrice <= bidPrice {
            askPrice = bidPrice + rand.Float64()*10 + 5 // Bid + 5 to 15
        }

        // Round prices to two decimal places
        bidPrice = math.Round(bidPrice*100) / 100
        askPrice = math.Round(askPrice*100) / 100

        // Generate PriceData with timestamp
        data := PriceData{
            Timestamp: time.Now().UTC().Format(time.RFC3339Nano),
            Bid:       bidPrice,
            Ask:       askPrice,
        }

        return data, interval
    }
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
