import time
import base64
import aiohttp
import asyncio

from src.data_fetching import __version__

class BaseAsyncRESTClient:
    def __init__(self, base_url, api_key=None, retry_attempts=3, retry_wait=1, retry_codes=None):
        self.base_url = base_url
        self.api_key = api_key
        self.retry_attempts = retry_attempts
        self.retry_wait = retry_wait
        self.retry_codes = retry_codes if retry_codes is not None else [429, 504]


        
    def _get_default_headers(self) -> dict:
        """
        Returns a dict with some default headers set; ie AUTH headers and such that should be useful on all requests
        Extracted for cases when using the default request functions are insufficient

        Returns:
            dict: The resulting dict of headers
        """
        headers = self._get_auth_headers()

        headers["User-Agent"] = "APCA-PY/" + __version__

        return headers
    
    def _get_auth_headers(self) -> dict:
        """
        Get the auth headers for a request. Meant to be overridden in clients that don't use this format for requests,
        ie: BrokerClient

        Returns:
            dict: A dict containing the expected auth headers
        """

        headers = {}

        if self._oauth_token:
            headers["Authorization"] = "Bearer " + self._oauth_token
        elif self._use_basic_auth:
            api_key_secret = "{key}:{secret}".format(
                key=self._api_key, secret=self._secret_key
            ).encode("utf-8")
            encoded_api_key_secret = base64.b64encode(api_key_secret).decode("utf-8")
            headers["Authorization"] = "Basic " + encoded_api_key_secret
        else:
            headers["APCA-API-KEY-ID"] = self._api_key
            headers["APCA-API-SECRET-KEY"] = self._secret_key

        return headers
    
    async def _request(self, method, path, data=None):
        url = f"{self.base_url}/{path}"
        headers = {"Authorization": f"Bearer {self.api_key}"} if self.api_key else {}
        params = data if method == "GET" else None
        json = data if method != "GET" else None
        
        async with aiohttp.ClientSession() as session:
            for attempt in range(self.retry_attempts + 1):
                try:
                    async with session.request(method, url, headers=headers, params=params, json=json) as response:
                        response.raise_for_status()
                        return await response.json()
                except aiohttp.ClientResponseError as e:
                    if e.status in self.retry_codes and attempt < self.retry_attempts:
                        await asyncio.sleep(self.retry_wait * (2 ** attempt))
                    else:
                        raise
                except aiohttp.ClientError as e:
                    if attempt < self.retry_attempts:
                        await asyncio.sleep(self.retry_wait * (2 ** attempt))
                    else:
                        raise
                    
    
# Usage in an async function
async def main():
    client = BaseAsyncRESTClient("https://api.example.com", "your_api_key")
    try:
        data = await client._request("GET", "data/endpoint")
        print(data)
    except Exception as e:
        print(f"Failed to fetch data: {e}")

if __name__ == "__main__":
    asyncio.run(main())
