-- RenovEasy Database Performance Monitoring and Optimization Queries
-- Description: Collection of SQL queries for monitoring and optimizing database performance
-- Date: 2025-08-20

-- =================================================================================
-- 1. DATABASE HEALTH MONITORING
-- =================================================================================

-- Check database connection status
SELECT 
    VARIABLE_NAME,
    VARIABLE_VALUE
FROM performance_schema.global_status 
WHERE VARIABLE_NAME IN (
    'Connections',
    'Max_used_connections', 
    'Threads_connected',
    'Threads_running',
    'Aborted_connects',
    'Connection_errors_max_connections'
);

-- Monitor database size and growth
SELECT 
    TABLE_SCHEMA as 'Database',
    ROUND(SUM(DATA_LENGTH + INDEX_LENGTH) / 1024 / 1024, 2) AS 'Database Size (MB)',
    ROUND(SUM(DATA_LENGTH) / 1024 / 1024, 2) AS 'Data Size (MB)',
    ROUND(SUM(INDEX_LENGTH) / 1024 / 1024, 2) AS 'Index Size (MB)'
FROM information_schema.TABLES 
WHERE TABLE_SCHEMA = 'renoveasy'
GROUP BY TABLE_SCHEMA;

-- Monitor table sizes and record counts
SELECT 
    TABLE_NAME,
    TABLE_ROWS as 'Estimated Rows',
    ROUND((DATA_LENGTH + INDEX_LENGTH) / 1024 / 1024, 2) AS 'Size (MB)',
    ROUND(DATA_LENGTH / 1024 / 1024, 2) AS 'Data (MB)',
    ROUND(INDEX_LENGTH / 1024 / 1024, 2) AS 'Index (MB)',
    ROUND(INDEX_LENGTH / DATA_LENGTH, 2) AS 'Index Ratio'
FROM information_schema.TABLES
WHERE TABLE_SCHEMA = 'renoveasy'
    AND TABLE_TYPE = 'BASE TABLE'
ORDER BY (DATA_LENGTH + INDEX_LENGTH) DESC;

-- =================================================================================
-- 2. SLOW QUERY ANALYSIS
-- =================================================================================

-- Enable slow query logging (if not already enabled)
-- SET GLOBAL slow_query_log = 'ON';
-- SET GLOBAL long_query_time = 2;
-- SET GLOBAL log_queries_not_using_indexes = 'ON';

-- Analyze slow queries from performance schema
SELECT 
    DIGEST_TEXT as 'Query Pattern',
    COUNT_STAR as 'Execution Count',
    ROUND(AVG_TIMER_WAIT/1000000000000, 3) as 'Avg Time (sec)',
    ROUND(MAX_TIMER_WAIT/1000000000000, 3) as 'Max Time (sec)',
    ROUND(SUM_ROWS_EXAMINED/COUNT_STAR, 0) as 'Avg Rows Examined',
    ROUND(SUM_ROWS_SENT/COUNT_STAR, 0) as 'Avg Rows Sent',
    FIRST_SEEN,
    LAST_SEEN
FROM performance_schema.events_statements_summary_by_digest 
WHERE SCHEMA_NAME = 'renoveasy'
    AND AVG_TIMER_WAIT > 1000000000000 -- More than 1 second average
ORDER BY AVG_TIMER_WAIT DESC
LIMIT 20;

-- Find queries that examine many rows but return few
SELECT 
    DIGEST_TEXT as 'Query Pattern',
    COUNT_STAR as 'Executions',
    ROUND(SUM_ROWS_EXAMINED/COUNT_STAR, 0) as 'Avg Rows Examined',
    ROUND(SUM_ROWS_SENT/COUNT_STAR, 0) as 'Avg Rows Sent',
    ROUND((SUM_ROWS_EXAMINED/COUNT_STAR)/(SUM_ROWS_SENT/COUNT_STAR), 2) as 'Examine/Send Ratio'
FROM performance_schema.events_statements_summary_by_digest 
WHERE SCHEMA_NAME = 'renoveasy'
    AND SUM_ROWS_SENT > 0
    AND (SUM_ROWS_EXAMINED/SUM_ROWS_SENT) > 100
ORDER BY (SUM_ROWS_EXAMINED/SUM_ROWS_SENT) DESC
LIMIT 10;

-- =================================================================================
-- 3. INDEX USAGE ANALYSIS
-- =================================================================================

-- Check for unused indexes
SELECT 
    object_schema as 'Database',
    object_name as 'Table',
    index_name as 'Index',
    count_star as 'Uses',
    count_read as 'Reads',
    count_write as 'Writes'
FROM performance_schema.table_io_waits_summary_by_index_usage 
WHERE object_schema = 'renoveasy'
    AND index_name IS NOT NULL 
    AND index_name != 'PRIMARY'
    AND count_star = 0
ORDER BY object_name, index_name;

-- Index efficiency analysis
SELECT 
    object_schema as 'Database',
    object_name as 'Table',
    index_name as 'Index',
    count_star as 'Total Uses',
    count_read as 'Reads',
    count_write as 'Writes',
    ROUND(count_read/count_star * 100, 2) as 'Read %'
FROM performance_schema.table_io_waits_summary_by_index_usage 
WHERE object_schema = 'renoveasy'
    AND index_name IS NOT NULL 
    AND count_star > 0
ORDER BY count_star DESC;

-- Table scan analysis (tables accessed without using indexes)
SELECT 
    object_schema as 'Database',
    object_name as 'Table',
    count_read as 'Reads',
    count_write as 'Writes',
    count_fetch as 'Fetches',
    ROUND(SUM_TIMER_FETCH/1000000000000, 3) as 'Fetch Time (sec)'
FROM performance_schema.table_io_waits_summary_by_table 
WHERE object_schema = 'renoveasy'
    AND count_fetch > 0
ORDER BY count_fetch DESC;

-- =================================================================================
-- 4. QUERY OPTIMIZATION RECOMMENDATIONS
-- =================================================================================

-- Find tables that might benefit from partitioning (large tables with time-based queries)
SELECT 
    TABLE_NAME,
    TABLE_ROWS as 'Estimated Rows',
    ROUND((DATA_LENGTH + INDEX_LENGTH) / 1024 / 1024, 2) AS 'Size (MB)',
    CREATE_TIME
FROM information_schema.TABLES
WHERE TABLE_SCHEMA = 'renoveasy'
    AND TABLE_ROWS > 100000  -- Tables with more than 100k rows
    AND TABLE_NAME IN ('orders', 'messages', 'user_analytics', 'auth_audit_log')
ORDER BY TABLE_ROWS DESC;

-- Analyze cardinality of indexed columns
SELECT 
    t.TABLE_NAME,
    s.COLUMN_NAME,
    s.CARDINALITY,
    ROUND(s.CARDINALITY / t.TABLE_ROWS * 100, 2) as 'Selectivity %'
FROM information_schema.TABLES t
JOIN information_schema.STATISTICS s ON t.TABLE_NAME = s.TABLE_NAME
WHERE t.TABLE_SCHEMA = 'renoveasy' 
    AND s.TABLE_SCHEMA = 'renoveasy'
    AND t.TABLE_ROWS > 1000
    AND s.CARDINALITY IS NOT NULL
ORDER BY t.TABLE_NAME, s.CARDINALITY DESC;

-- =================================================================================
-- 5. SPECIFIC PERFORMANCE QUERIES FOR RENOVEEASY
-- =================================================================================

-- Monitor order creation and processing performance
SELECT 
    DATE(created_at) as 'Date',
    COUNT(*) as 'Orders Created',
    COUNT(CASE WHEN status = 'completed' THEN 1 END) as 'Completed',
    COUNT(CASE WHEN status = 'canceled' THEN 1 END) as 'Canceled',
    AVG(CASE WHEN completed_at IS NOT NULL AND assigned_at IS NOT NULL 
        THEN TIMESTAMPDIFF(HOUR, assigned_at, completed_at) END) as 'Avg Completion Hours'
FROM orders 
WHERE created_at >= DATE_SUB(NOW(), INTERVAL 30 DAY)
GROUP BY DATE(created_at)
ORDER BY DATE(created_at) DESC;

-- Analyze geographic distribution of orders and workers
SELECT 
    city,
    COUNT(*) as 'Total Orders',
    COUNT(CASE WHEN status = 'published' THEN 1 END) as 'Available Orders',
    AVG(budget_max - budget_min) as 'Avg Budget Range'
FROM orders 
WHERE created_at >= DATE_SUB(NOW(), INTERVAL 7 DAY)
GROUP BY city
ORDER BY COUNT(*) DESC
LIMIT 10;

-- Worker performance analysis
SELECT 
    u.id as worker_id,
    up.display_name,
    up.preferred_city,
    ws.total_orders,
    ws.completed_orders,
    ws.average_rating,
    ws.total_earnings,
    COUNT(ob.id) as 'Active Bids'
FROM users u
JOIN user_profiles up ON u.id = up.user_id
JOIN worker_stats ws ON u.id = ws.worker_id
LEFT JOIN order_bids ob ON u.id = ob.worker_id AND ob.status = 'pending'
WHERE u.user_type = 'worker'
    AND ws.total_orders > 0
GROUP BY u.id
ORDER BY ws.average_rating DESC, ws.total_orders DESC
LIMIT 20;

-- Message traffic analysis
SELECT 
    DATE(created_at) as 'Date',
    COUNT(*) as 'Messages Sent',
    COUNT(CASE WHEN is_read = TRUE THEN 1 END) as 'Messages Read',
    ROUND(COUNT(CASE WHEN is_read = TRUE THEN 1 END) / COUNT(*) * 100, 2) as 'Read Rate %',
    COUNT(DISTINCT sender_id) as 'Active Senders',
    COUNT(DISTINCT receiver_id) as 'Active Receivers'
FROM messages 
WHERE created_at >= DATE_SUB(NOW(), INTERVAL 7 DAY)
GROUP BY DATE(created_at)
ORDER BY DATE(created_at) DESC;

-- =================================================================================
-- 6. CACHE PERFORMANCE MONITORING
-- =================================================================================

-- Query cache performance (Note: Query cache is removed in MySQL 8.0)
-- For MySQL 5.7 and earlier:
-- SELECT 
--     VARIABLE_NAME,
--     VARIABLE_VALUE
-- FROM performance_schema.global_status 
-- WHERE VARIABLE_NAME LIKE 'Qcache%';

-- Buffer pool usage
SELECT 
    VARIABLE_NAME,
    VARIABLE_VALUE,
    CASE 
        WHEN VARIABLE_NAME = 'Innodb_buffer_pool_size' THEN ROUND(VARIABLE_VALUE/1024/1024/1024, 2)
        WHEN VARIABLE_NAME LIKE '%pages%' THEN VARIABLE_VALUE
        ELSE ROUND(VARIABLE_VALUE/1024/1024, 2)
    END as 'Formatted Value'
FROM performance_schema.global_status 
WHERE VARIABLE_NAME IN (
    'Innodb_buffer_pool_size',
    'Innodb_buffer_pool_pages_total',
    'Innodb_buffer_pool_pages_free',
    'Innodb_buffer_pool_pages_data',
    'Innodb_buffer_pool_pages_dirty',
    'Innodb_buffer_pool_read_requests',
    'Innodb_buffer_pool_reads'
);

-- Calculate buffer pool hit ratio
SELECT 
    ROUND((1 - (reads.VARIABLE_VALUE / read_requests.VARIABLE_VALUE)) * 100, 2) as 'Buffer Pool Hit Ratio %'
FROM 
    (SELECT VARIABLE_VALUE FROM performance_schema.global_status WHERE VARIABLE_NAME = 'Innodb_buffer_pool_reads') reads,
    (SELECT VARIABLE_VALUE FROM performance_schema.global_status WHERE VARIABLE_NAME = 'Innodb_buffer_pool_read_requests') read_requests;

-- =================================================================================
-- 7. MAINTENANCE AND OPTIMIZATION COMMANDS
-- =================================================================================

-- Optimize tables (run during maintenance window)
-- OPTIMIZE TABLE orders, messages, user_analytics, auth_audit_log;

-- Analyze tables to update statistics
-- ANALYZE TABLE users, orders, order_bids, reviews, messages;

-- Check table integrity
-- CHECK TABLE users, orders, order_bids, reviews, messages;

-- Update table statistics
-- SET GLOBAL innodb_stats_on_metadata = ON;

-- =================================================================================
-- 8. AUTOMATED PERFORMANCE REPORTS
-- =================================================================================

-- Create a view for daily performance summary
CREATE OR REPLACE VIEW daily_performance_summary AS
SELECT 
    DATE(NOW()) as report_date,
    (SELECT COUNT(*) FROM orders WHERE DATE(created_at) = DATE(NOW())) as orders_today,
    (SELECT COUNT(*) FROM users WHERE DATE(created_at) = DATE(NOW())) as new_users_today,
    (SELECT COUNT(*) FROM messages WHERE DATE(created_at) = DATE(NOW())) as messages_today,
    (SELECT COUNT(*) FROM order_bids WHERE DATE(created_at) = DATE(NOW())) as bids_today,
    (SELECT AVG(TIMESTAMPDIFF(MINUTE, created_at, assigned_at)) FROM orders 
     WHERE DATE(assigned_at) = DATE(NOW()) AND assigned_at IS NOT NULL) as avg_assignment_time_minutes,
    (SELECT COUNT(DISTINCT user_id) FROM user_analytics 
     WHERE DATE(created_at) = DATE(NOW())) as daily_active_users;

-- Create a view for problematic queries
CREATE OR REPLACE VIEW slow_query_summary AS
SELECT 
    LEFT(DIGEST_TEXT, 100) as query_snippet,
    COUNT_STAR as executions,
    ROUND(AVG_TIMER_WAIT/1000000000000, 3) as avg_time_sec,
    ROUND(MAX_TIMER_WAIT/1000000000000, 3) as max_time_sec,
    FIRST_SEEN,
    LAST_SEEN
FROM performance_schema.events_statements_summary_by_digest 
WHERE SCHEMA_NAME = 'renoveasy'
    AND AVG_TIMER_WAIT > 2000000000000 -- More than 2 seconds average
ORDER BY AVG_TIMER_WAIT DESC;

-- =================================================================================
-- 9. ALERTING QUERIES (FOR MONITORING SYSTEMS)
-- =================================================================================

-- High connection usage alert (>80% of max connections)
SELECT 
    CASE 
        WHEN (threads_connected.VARIABLE_VALUE / max_connections.VARIABLE_VALUE) > 0.8 
        THEN 'ALERT: High connection usage'
        ELSE 'OK'
    END as connection_status,
    CONCAT(ROUND((threads_connected.VARIABLE_VALUE / max_connections.VARIABLE_VALUE) * 100, 2), '%') as usage_percentage
FROM 
    (SELECT VARIABLE_VALUE FROM performance_schema.global_variables WHERE VARIABLE_NAME = 'max_connections') max_connections,
    (SELECT VARIABLE_VALUE FROM performance_schema.global_status WHERE VARIABLE_NAME = 'Threads_connected') threads_connected;

-- Long running transactions alert
SELECT 
    trx_id,
    trx_state,
    trx_started,
    TIMESTAMPDIFF(SECOND, trx_started, NOW()) as duration_seconds,
    trx_mysql_thread_id,
    trx_query
FROM information_schema.INNODB_TRX 
WHERE TIMESTAMPDIFF(SECOND, trx_started, NOW()) > 300 -- Longer than 5 minutes
ORDER BY trx_started;

-- Deadlock detection
SELECT 
    VARIABLE_VALUE as deadlock_count
FROM performance_schema.global_status 
WHERE VARIABLE_NAME = 'Innodb_deadlocks';

-- =================================================================================
-- 10. SAMPLE OPTIMIZATION STRATEGIES
-- =================================================================================

-- Example: Optimize order search query
-- Before optimization:
-- SELECT * FROM orders o
-- JOIN users u ON o.customer_id = u.id  
-- WHERE o.city = '北京' AND o.status = 'published'
-- ORDER BY o.created_at DESC;

-- After optimization (with proper indexing):
-- CREATE INDEX idx_orders_city_status_created ON orders(city, status, created_at DESC);
-- The query will now use the composite index efficiently

-- Example: Optimize worker search by location
-- For geographic queries, consider using spatial indexes:
-- CREATE SPATIAL INDEX idx_user_profiles_location ON user_profiles(POINT(longitude, latitude));

-- Example: Optimize message pagination
-- Instead of OFFSET/LIMIT (slow for large offsets):
-- SELECT * FROM messages WHERE conversation_id = ? ORDER BY created_at DESC LIMIT 20;
-- Use cursor-based pagination:
-- SELECT * FROM messages WHERE conversation_id = ? AND created_at < ? ORDER BY created_at DESC LIMIT 20;

DELIMITER //
CREATE PROCEDURE GetPerformanceReport()
BEGIN
    -- Daily summary
    SELECT 'DAILY SUMMARY' as section;
    SELECT * FROM daily_performance_summary;
    
    -- Top slow queries
    SELECT 'SLOW QUERIES' as section;
    SELECT * FROM slow_query_summary LIMIT 10;
    
    -- Database size info
    SELECT 'DATABASE SIZE' as section;
    SELECT 
        TABLE_NAME,
        TABLE_ROWS,
        ROUND((DATA_LENGTH + INDEX_LENGTH) / 1024 / 1024, 2) AS 'Size_MB'
    FROM information_schema.TABLES
    WHERE TABLE_SCHEMA = 'renoveasy'
        AND TABLE_TYPE = 'BASE TABLE'
    ORDER BY (DATA_LENGTH + INDEX_LENGTH) DESC
    LIMIT 10;
    
    -- Connection status
    SELECT 'CONNECTION STATUS' as section;
    SELECT 
        VARIABLE_NAME,
        VARIABLE_VALUE
    FROM performance_schema.global_status 
    WHERE VARIABLE_NAME IN ('Threads_connected', 'Max_used_connections');
    
END //
DELIMITER ;

-- Run the performance report
-- CALL GetPerformanceReport();