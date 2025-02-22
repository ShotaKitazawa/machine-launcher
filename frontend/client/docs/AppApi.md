# \AppApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**list_servers**](AppApi.md#list_servers) | **GET** /api/servers | List Servers
[**start_server**](AppApi.md#start_server) | **PUT** /api/servers/start | Start server
[**stop_server**](AppApi.md#stop_server) | **PUT** /api/servers/stop | Stop server



## list_servers

> Vec<models::Server> list_servers()
List Servers

### Parameters

This endpoint does not need any parameter.

### Return type

[**Vec<models::Server>**](Server.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## start_server

> models::Server start_server(server_name)
Start server

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**server_name** | [**ServerName**](ServerName.md) |  | [required] |

### Return type

[**models::Server**](Server.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## stop_server

> models::Server stop_server(server_name)
Stop server

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**server_name** | [**ServerName**](ServerName.md) |  | [required] |

### Return type

[**models::Server**](Server.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

