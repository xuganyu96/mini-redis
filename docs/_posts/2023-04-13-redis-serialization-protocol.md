---
layout: post
title:  "Redis 序列化协议"
date:   2023-04-13 21:50:00 -0700
---

REdis Serialization Protocol (RESP) 构建在 TCP 协议之上，默认端口 6379。通常情况下，沟通由客户端发送命令发起，然后由服务器向客户端发送回复。Pipelining（客户端一次发送多个命令）和 pub/sub 这两个功能的沟通方式更加复杂，暂时略过

# 数据类型
RESP 一共有五种数据类型：

## Simple String
- 由 `+` 开头，由 CRLF 结尾
- 二进制不安全：字符串不能带有 CR 或者 LF

## Error
- 类似 Simple String，但是由 `-` 开头
- 二进制不安全
- 用于传达错误信息，比如指令不正确

## Integer
- 类似 Simple String，但是由 `:` 开头
- 整数数值保证在64位有符号整数范围之内
- e.g. `DEL key1 key2 key3` 返还被删除的键数

## Bulk String
- 格式如`$<n><CRLF><string><CRLF>`，其中第一个 n 代表字符串的长度
- 二进制安全
- `$0\r\n\r\n` 代表空字符串
- `$-1\r\n` 代表 Null

## Array
- 格式如 `*<n><CRLF>`， 其中 n 记录元素的数目
- Array 可以嵌套
- Array 常用于发送指令，比如 `SET 古尔丹 代价是什么` 其实是一个三个元素的指令

```
*3\r\n+SET\r\n$15\r\n那么古尔丹\r\n$18\r\n代价是什么呢\r\n
|--1--|---2---|---------3--------|----------4---------|

1. 定义一个3个元素的序列
2. 第一个元素是 SimpleString("SET")
3. 第二个元素是 BulkString("那么古尔丹")，字符串长度 15 字节
4. 第三个元素是 BulkString("代价是什么呢")，字符串长度 18 字节
```