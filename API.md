# API Documentation

This document provides detailed reference documentation for the TulparAPI endpoints.

## Base URL

All API endpoints are prefixed with `/api/v1`.

```text
http://localhost:3000/api/v1
```

## Endpoints

### 1. List Collections

Retrieves a list of all available content collections. This endpoint returns metadata only, not the full content.

- **URL:** `/content/collections`
- **Method:** `GET`
- **Success Response:**
  - **Code:** 200 OK
  - **Content:**
    ```json
    [
      {
        "id": 1,
        "alias": "rubaiyat",
        "name": "Rubaiyat of Omar Khayyam",
        "file_path": "storage/collections/rubaiyat.json.lz4",
        "language": "en"
      }
    ]
    ```

### 2. Get Collection

Retrieves the full content of a specific collection. If the collection is not in the memory cache, it is fetched from the compressed storage, decompressed, and then cached for subsequent requests.

- **URL:** `/content/collections/:alias`
- **Method:** `GET`
- **URL Parameters:**
  - `alias` (string): The unique alias/key of the collection (e.g., `rubaiyat`).
- **Success Response:**
  - **Code:** 200 OK
  - **Content:**
    ```json
    {
      "alias": "rubaiyat",
      "name": "Rubaiyat of Omar Khayyam",
      "items": [
        {
          "id": "1",
          "title": "Quatrain I",
          "body": "Awake! for Morning in the Bowl of Night..."
        },
        {
          "id": "2",
          "title": "Quatrain II",
          "body": "Dreaming when Dawn's Left Hand was in the Sky..."
        }
        // ... more items
      ]
    }
    ```
- **Error Response:**
  - **Code:** 404 Not Found
  - **Content:** `Collection not found`

### 3. Get Collection Item

Retrieves a specific item from a collection directly. This avoids fetching the entire collection if only a single item is needed, though the underlying system may still load the full collection into cache for performance optimization.

- **URL:** `/content/collections/:alias/items/:item_id`
- **Method:** `GET`
- **URL Parameters:**
  - `alias` (string): The unique alias/key of the collection.
  - `item_id` (string): The unique ID of the specific item within the collection.
- **Success Response:**
  - **Code:** 200 OK
  - **Content:**
    ```json
    {
      "id": "1",
      "title": "Quatrain I",
      "body": "Awake! for Morning in the Bowl of Night\nHas flung the Stone that puts the Stars to Flight:\nAnd Lo! the Hunter of the East has caught\nThe Sultan's Turret in a Noose of Light."
    }
    ```
- **Error Response:**
  - **Code:** 404 Not Found
  - **Content:** `Item not found`

## Middleware & Headers

### Request Tracking
- **Header:** `x-request-id`
- **Description:** Every response includes a `x-request-id` header containing a unique UUID. Use this ID when reporting issues or searching through server logs.

### Rate Limiting
- **Limit:** 2 requests per second.
- **Burst:** 5 requests.
- **Error:** Requests exceeding this limit will receive a `429 Too Many Requests` status code.

## Error Handling

The API uses standard HTTP status codes to indicate the success or failure of an API request.

- **200 OK:** The request was successful.
- **429 Too Many Requests:** Rate limit exceeded.
- **404 Not Found:** The requested resource (collection or item) could not be found.
- **500 Internal Server Error:** An unexpected error occurred on the server (e.g., database connection issue, file decompression error).
