package main

import (
	// "encoding/base64"
	"fmt"
	"github.com/panjf2000/ants/v2"
	"io"
	"net/http"
	"os"
	"strconv"
	"sync"
	"sync/atomic"
	"time"
)

const (
	baseURL   = "https://data.alpaca.markets/v2/stocks"
	userAgent = "APCA-GO/v3.4.0"
)

// Client holds the configuration for the API client
type Client struct {
	BaseURL       string
	APIKey        string
	APISecret     string
	RetryAttempts int
	RetryWait     int
	RetryCodes    map[int]bool
	httpClient    *http.Client
}

// NewClient creates a new API client
func NewClient() *Client {
	return &Client{
		BaseURL:       baseURL,
		APIKey:        os.Getenv("APCA_API_KEY_ID"),
		APISecret:     os.Getenv("APCA_API_SECRET_KEY"),
		RetryAttempts: 3,
		RetryWait:     1,
		RetryCodes:    map[int]bool{429: true, 504: true},
		httpClient:    &http.Client{},
	}
}

// // getAuthHeaders builds the authentication headers
// func (c *Client) getAuthHeaders() map[string]string {
// 	return map[string]string{
// 		"APCA-API-KEY-ID":     c.APIKey,
// 		"APCA-API-SECRET-KEY": c.APISecret,
// 		"User-Agent":          userAgent,
// 	}
// }

// makeRequest makes an HTTP request handling retries
func (c *Client) makeRequest(method, path string) (string, error) {
	url := c.BaseURL + "/" + path
	req, err := http.NewRequest(method, url, nil)
	if err != nil {
		return "", fmt.Errorf("error creating request: %w", err)
	}

	req.Header.Set("User-Agent", userAgent)
	req.Header.Set("APCA-API-KEY-ID", c.APIKey)
	req.Header.Set("APCA-API-SECRET-KEY", c.APISecret)

	// retryAttempts := 3
	retryWait := c.RetryWait

	for i := 0; i <= c.RetryAttempts; i++ {
		resp, err := c.httpClient.Do(req)
		if err != nil {
			return "", fmt.Errorf("error executing request: %w", err)
		}
		defer resp.Body.Close()

		if resp.StatusCode == http.StatusOK {
			body, err := io.ReadAll(resp.Body)
			if err != nil {
				return "", fmt.Errorf("error reading response: %w", err)
			}
			return string(body), nil
		} else if resp.StatusCode == 429 {
			retryAfter := resp.Header.Get("Retry-After")
			waitTime, parseErr := strconv.Atoi(retryAfter)
			if parseErr == nil {
				time.Sleep(time.Duration(waitTime) * time.Second)
			} else {
				// Exponential backoff if Retry-After is not available
				time.Sleep(time.Duration(retryWait) * time.Second)
				retryWait *= 2
			}
		} else {
			return "", fmt.Errorf("failed with status code %d", resp.StatusCode)
		}
	}
	return "", fmt.Errorf("retries exceeded ")
}

func fetchData(client *Client, path string, wg *sync.WaitGroup, calls *int32) {
	defer wg.Done()
	_, err := client.makeRequest("GET", path)
	if err != nil {
		fmt.Println("Error fetching data:", err)
		return
	}
	atomic.AddInt32(calls, 1)
}

func main() {
	client := NewClient()
	var calls int32
	path := "bars?symbols=NVDA&timeframe=1Day&start=2016-01-03T00:00:00Z&end=2022-01-04T00:00:00Z&limit=1000&adjustment=all&feed=sip&sort=asc"

	// Initialize the ants pool with the correct function signature
	p, _ := ants.NewPoolWithFunc(5, func(i interface{}) {
		fetchData(client, path, i.(*sync.WaitGroup), &calls)
	})
	defer p.Release()

	var wg sync.WaitGroup
	timer := time.NewTimer(time.Minute)
	for {
		select {
		case <-timer.C:
			wg.Wait()
			fmt.Printf("Total API calls made in one minute: %d\n", calls)
			return
		default:
			wg.Add(1)
			_ = p.Invoke(&wg)
		}
		time.Sleep(10 * time.Millisecond) // Control the rate of invoking go routines
	}

}
