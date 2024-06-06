package main

import (
	"encoding/json"
	"fmt"
	"github.com/alpacahq/alpaca-trade-api-go/v3/marketdata"
	"log"
	"os"
	"time"
)

// Function to set up a logger
func setupLogger() (*os.File, error) {
	// Generate the log file name
	timestamp := time.Now().Format("20060102_150405")
	logFileName := fmt.Sprintf("./logs/go_alpaca_fetch_%s.log", timestamp)

	// Open the log file
	logFile, err := os.OpenFile(logFileName, os.O_APPEND|os.O_CREATE|os.O_WRONLY, 0644)
	if err != nil {
		return nil, err
	}

	// Set the output of the logger
	log.SetOutput(logFile)

	return logFile, nil
}

// Initialize and return a marketdata Client
func initClient() *marketdata.Client {
	// Retrieve API keys from environment variables
	apiKey := os.Getenv("APCA_API_KEY_ID")
	apiSecret := os.Getenv("APCA_API_SECRET_KEY")
	if apiKey == "" || apiSecret == "" {
		log.Fatal("API credentials are not set")
	}

	// Create the Alpaca market data client
	clientOpts := marketdata.ClientOpts{
		APIKey:    apiKey,
		APISecret: apiSecret,
		BaseURL:   "https://data.alpaca.markets",
	}
	client := marketdata.NewClient(clientOpts)
	return client
}

// Get bars for one stock
func fetchMarketBars(client *marketdata.Client, symbol string, timeframe marketdata.TimeFrame, start_time time.Time, end_time time.Time) []marketdata.Bar {

	bars, err := client.GetBars(symbol, marketdata.GetBarsRequest{
		TimeFrame: timeframe,
		Start:     start_time,
		End:       end_time,
	})

	if err != nil {
		log.Printf("Error fetching bars: %v", err)
	}

	return bars

}

func writeBarsToJSON(bars []marketdata.Bar, location string) {
	// Marshal the bars into JSON
	barsJSON, err := json.Marshal(bars)
	if err != nil {
		log.Printf("Error marshaling bars to JSON: %v", err)
		return
	}

	// Write the JSON to the specified location
	err = os.WriteFile(location, barsJSON, 0644)
	if err != nil {
		log.Printf("Error writing JSON to file: %v", err)
	}
}

func TestFetchAndWriteBars() {
	// Initialize the market data client
	client := initClient()

	// Start the timer
	start := time.Now()

	// Initialize the successful call counter
	successfulCalls := 0

	for {
		// Check if one minute has passed
		if time.Since(start) >= time.Minute {
			break
		}

		// Fetch bars
		end_date := time.Now().Add(-1 * time.Hour * 24)
		// Starts from 2016, 1, 1
		start_date := time.Date(2016, 1, 1, 0, 0, 0, 0, time.UTC)

		bars := fetchMarketBars(client, "NVDA", marketdata.OneDay, start_date, end_date)

		// If bars were fetched successfully, write them to a JSON file
		if len(bars) > 0 {
			writeBarsToJSON(bars, fmt.Sprintf("./data/bars%d.json", successfulCalls))
			successfulCalls++
		}

	}

	// Print the number of successful calls
	log.Printf("Made %d successful calls in one minute", successfulCalls)
}

func main() {
	// Set up the logger
	logFile, err := setupLogger()
	if err != nil {
		log.Fatalf("Error setting up logger: %v", err)
	}
	defer logFile.Close()

	// Test fetching and writing bars
	TestFetchAndWriteBars()

	// // Initialize the market data client
	// client := initClient()

	// // Get bars for one stock
	// bars := fetchMarketBars(client, "AAPL", marketdata.OneMin, time.Now().Add(-24*time.Hour), time.Now().Add(-1*time.Hour))

	// // Write the bars to a JSON file
	// writeBarsToJSON(bars, "./data/aapl_bars.json")

	// log.Printf("Fetched %d bars for AAPL\n", len(bars))

}

// func fetchBars(symbol string, start time.Time, end time.Time, duration time.Duration) {

// 	// Prepare to measure the number of requests
// 	startTime := time.Now()
// 	endTime := startTime.Add(duration)
// 	calls := 0

// 	// Fetch bars in a loop for the specified duration
// 	for time.Now().Before(endTime) {
// 		bars, err := client.GetBars(symbol, marketdata.GetBarsRequest{
// 			TimeFrame: marketdata.OneDay,
// 			Start:     start,
// 			End:       end,
// 		})
// 		if err != nil {
// 			log.Printf("Error fetching bars for %s: %v\n", symbol, err)
// 			continue
// 		}
// 		calls++
// 		fmt.Printf("Fetched %d bars for %s\n", len(bars), symbol)
// 	}

// 	fmt.Printf("Total API calls made in %v minutes: %d\n", duration.Minutes(), calls)
// }
