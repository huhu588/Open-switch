// Package repository 提供内存缓存适配器，替代 Redis 用于桌面部署场景。
// 使用 sync.Map + 定时清理实现，适用于单实例桌面应用。
package repository

import (
	"context"
	"sync"
	"time"
)

// MemoryCache 基于内存的缓存实现，替代 Redis
type MemoryCache struct {
	data    sync.Map
	mu      sync.RWMutex
	stopCh  chan struct{}
}

type cacheEntry struct {
	value     interface{}
	expiresAt time.Time
	hasExpiry bool
}

// NewMemoryCache 创建内存缓存实例
func NewMemoryCache() *MemoryCache {
	mc := &MemoryCache{
		stopCh: make(chan struct{}),
	}
	go mc.cleanupLoop()
	return mc
}

func (mc *MemoryCache) cleanupLoop() {
	ticker := time.NewTicker(60 * time.Second)
	defer ticker.Stop()
	for {
		select {
		case <-ticker.C:
			now := time.Now()
			mc.data.Range(func(key, value interface{}) bool {
				entry, ok := value.(*cacheEntry)
				if ok && entry.hasExpiry && now.After(entry.expiresAt) {
					mc.data.Delete(key)
				}
				return true
			})
		case <-mc.stopCh:
			return
		}
	}
}

// Close 关闭缓存
func (mc *MemoryCache) Close() error {
	close(mc.stopCh)
	return nil
}

// Set 设置缓存值
func (mc *MemoryCache) Set(ctx context.Context, key string, value interface{}, expiration time.Duration) error {
	entry := &cacheEntry{
		value: value,
	}
	if expiration > 0 {
		entry.expiresAt = time.Now().Add(expiration)
		entry.hasExpiry = true
	}
	mc.data.Store(key, entry)
	return nil
}

// Get 获取缓存值
func (mc *MemoryCache) Get(ctx context.Context, key string) (interface{}, error) {
	val, ok := mc.data.Load(key)
	if !ok {
		return nil, ErrCacheMiss
	}
	entry := val.(*cacheEntry)
	if entry.hasExpiry && time.Now().After(entry.expiresAt) {
		mc.data.Delete(key)
		return nil, ErrCacheMiss
	}
	return entry.value, nil
}

// GetString 获取字符串缓存值
func (mc *MemoryCache) GetString(ctx context.Context, key string) (string, error) {
	val, err := mc.Get(ctx, key)
	if err != nil {
		return "", err
	}
	str, ok := val.(string)
	if !ok {
		return "", ErrCacheTypeMismatch
	}
	return str, nil
}

// Del 删除缓存
func (mc *MemoryCache) Del(ctx context.Context, keys ...string) error {
	for _, key := range keys {
		mc.data.Delete(key)
	}
	return nil
}

// Exists 检查缓存是否存在
func (mc *MemoryCache) Exists(ctx context.Context, key string) (bool, error) {
	_, err := mc.Get(ctx, key)
	if err != nil {
		return false, nil
	}
	return true, nil
}

// Incr 自增
func (mc *MemoryCache) Incr(ctx context.Context, key string) (int64, error) {
	mc.mu.Lock()
	defer mc.mu.Unlock()

	val, ok := mc.data.Load(key)
	if !ok {
		mc.data.Store(key, &cacheEntry{value: int64(1)})
		return 1, nil
	}
	entry := val.(*cacheEntry)
	if entry.hasExpiry && time.Now().After(entry.expiresAt) {
		mc.data.Store(key, &cacheEntry{value: int64(1)})
		return 1, nil
	}
	n, ok := entry.value.(int64)
	if !ok {
		mc.data.Store(key, &cacheEntry{value: int64(1), expiresAt: entry.expiresAt, hasExpiry: entry.hasExpiry})
		return 1, nil
	}
	n++
	entry.value = n
	mc.data.Store(key, entry)
	return n, nil
}

// Expire 设置过期时间
func (mc *MemoryCache) Expire(ctx context.Context, key string, expiration time.Duration) error {
	val, ok := mc.data.Load(key)
	if !ok {
		return nil
	}
	entry := val.(*cacheEntry)
	entry.expiresAt = time.Now().Add(expiration)
	entry.hasExpiry = true
	mc.data.Store(key, entry)
	return nil
}

// SetNX 如果不存在则设置
func (mc *MemoryCache) SetNX(ctx context.Context, key string, value interface{}, expiration time.Duration) (bool, error) {
	mc.mu.Lock()
	defer mc.mu.Unlock()

	existing, ok := mc.data.Load(key)
	if ok {
		entry := existing.(*cacheEntry)
		if !entry.hasExpiry || time.Now().Before(entry.expiresAt) {
			return false, nil
		}
	}
	entry := &cacheEntry{value: value}
	if expiration > 0 {
		entry.expiresAt = time.Now().Add(expiration)
		entry.hasExpiry = true
	}
	mc.data.Store(key, entry)
	return true, nil
}

// HSet Hash 设置
func (mc *MemoryCache) HSet(ctx context.Context, key, field string, value interface{}) error {
	mc.mu.Lock()
	defer mc.mu.Unlock()

	var hashMap map[string]interface{}
	val, ok := mc.data.Load(key)
	if ok {
		entry := val.(*cacheEntry)
		hashMap, _ = entry.value.(map[string]interface{})
	}
	if hashMap == nil {
		hashMap = make(map[string]interface{})
	}
	hashMap[field] = value
	mc.data.Store(key, &cacheEntry{value: hashMap})
	return nil
}

// HGet Hash 获取
func (mc *MemoryCache) HGet(ctx context.Context, key, field string) (interface{}, error) {
	val, ok := mc.data.Load(key)
	if !ok {
		return nil, ErrCacheMiss
	}
	entry := val.(*cacheEntry)
	hashMap, ok := entry.value.(map[string]interface{})
	if !ok {
		return nil, ErrCacheTypeMismatch
	}
	result, ok := hashMap[field]
	if !ok {
		return nil, ErrCacheMiss
	}
	return result, nil
}

// HDel Hash 删除
func (mc *MemoryCache) HDel(ctx context.Context, key string, fields ...string) error {
	mc.mu.Lock()
	defer mc.mu.Unlock()

	val, ok := mc.data.Load(key)
	if !ok {
		return nil
	}
	entry := val.(*cacheEntry)
	hashMap, ok := entry.value.(map[string]interface{})
	if !ok {
		return nil
	}
	for _, field := range fields {
		delete(hashMap, field)
	}
	return nil
}

// MemoryRateLimiter 基于内存的限流器
type MemoryRateLimiter struct {
	windows sync.Map
	mu      sync.Mutex
}

type rateLimitWindow struct {
	count     int64
	expiresAt time.Time
}

// NewMemoryRateLimiter 创建内存限流器
func NewMemoryRateLimiter() *MemoryRateLimiter {
	return &MemoryRateLimiter{}
}

// Allow 检查是否允许请求
func (rl *MemoryRateLimiter) Allow(ctx context.Context, key string, limit int64, window time.Duration) (bool, error) {
	rl.mu.Lock()
	defer rl.mu.Unlock()

	now := time.Now()
	val, ok := rl.windows.Load(key)
	if !ok || now.After(val.(*rateLimitWindow).expiresAt) {
		rl.windows.Store(key, &rateLimitWindow{
			count:     1,
			expiresAt: now.Add(window),
		})
		return true, nil
	}

	w := val.(*rateLimitWindow)
	if w.count >= limit {
		return false, nil
	}
	w.count++
	return true, nil
}

// 错误定义
var (
	ErrCacheMiss         = &CacheError{msg: "cache miss"}
	ErrCacheTypeMismatch = &CacheError{msg: "cache type mismatch"}
)

type CacheError struct {
	msg string
}

func (e *CacheError) Error() string {
	return e.msg
}
