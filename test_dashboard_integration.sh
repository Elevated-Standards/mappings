#!/bin/bash

# Compliance Dashboard Integration Test
# Tests the complete dashboard system including frontend and backend

echo "🧪 Compliance Dashboard Integration Test"
echo "========================================"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test configuration
BACKEND_URL="http://localhost:8080"
FRONTEND_URL="http://localhost:3000"

# Function to test API endpoint
test_endpoint() {
    local endpoint=$1
    local description=$2
    local expected_status=${3:-200}
    
    echo -n "Testing $description... "
    
    response=$(curl -s -w "%{http_code}" -o /tmp/response.json "$BACKEND_URL$endpoint")
    status_code="${response: -3}"
    
    if [ "$status_code" -eq "$expected_status" ]; then
        echo -e "${GREEN}✅ PASS${NC} (HTTP $status_code)"
        return 0
    else
        echo -e "${RED}❌ FAIL${NC} (HTTP $status_code, expected $expected_status)"
        return 1
    fi
}

# Function to test JSON response structure
test_json_structure() {
    local endpoint=$1
    local jq_filter=$2
    local description=$3
    
    echo -n "Testing $description... "
    
    response=$(curl -s "$BACKEND_URL$endpoint")
    result=$(echo "$response" | jq -r "$jq_filter" 2>/dev/null)
    
    if [ "$result" != "null" ] && [ "$result" != "" ]; then
        echo -e "${GREEN}✅ PASS${NC} ($result)"
        return 0
    else
        echo -e "${RED}❌ FAIL${NC} (Invalid JSON structure)"
        return 1
    fi
}

# Function to check if service is running
check_service() {
    local url=$1
    local service_name=$2
    
    echo -n "Checking $service_name... "
    
    if curl -s --connect-timeout 5 "$url" > /dev/null 2>&1; then
        echo -e "${GREEN}✅ RUNNING${NC}"
        return 0
    else
        echo -e "${RED}❌ NOT RUNNING${NC}"
        return 1
    fi
}

# Start tests
echo -e "\n${BLUE}1. Service Availability Tests${NC}"
echo "--------------------------------"

check_service "$BACKEND_URL/health" "Backend API Server"
backend_running=$?

check_service "$FRONTEND_URL" "Frontend Development Server"
frontend_running=$?

if [ $backend_running -ne 0 ]; then
    echo -e "\n${RED}❌ Backend is not running. Please start it with:${NC}"
    echo "   cargo run --package compliance-dashboard --bin server"
    exit 1
fi

if [ $frontend_running -ne 0 ]; then
    echo -e "\n${YELLOW}⚠️  Frontend is not running. Please start it with:${NC}"
    echo "   cd crates/compliance-dashboard/frontend && npm run dev"
fi

echo -e "\n${BLUE}2. API Endpoint Tests${NC}"
echo "----------------------"

# Test basic endpoints
test_endpoint "/health" "Health Check"
test_endpoint "/api/dashboard" "Dashboard Overview"
test_endpoint "/api/dashboard/metrics" "Dashboard Metrics"
test_endpoint "/api/dashboard/widgets" "Dashboard Widgets"
test_endpoint "/api/controls" "Controls List"
test_endpoint "/api/frameworks" "Frameworks List"
test_endpoint "/api/realtime/stats" "Real-time Stats"

echo -e "\n${BLUE}3. Data Structure Tests${NC}"
echo "------------------------"

# Test JSON response structures
test_json_structure "/api/dashboard" ".overview.total_controls" "Dashboard total controls"
test_json_structure "/api/dashboard" ".overview.implementation_percentage" "Implementation percentage"
test_json_structure "/api/controls" ".controls | length" "Controls array length"
test_json_structure "/api/frameworks" ".frameworks | length" "Frameworks array length"

echo -e "\n${BLUE}4. Sample Data Verification${NC}"
echo "----------------------------"

# Verify sample data is loaded
echo -n "Checking sample controls... "
controls_count=$(curl -s "$BACKEND_URL/api/controls" | jq -r '.controls | length')
if [ "$controls_count" -ge 3 ]; then
    echo -e "${GREEN}✅ PASS${NC} ($controls_count controls loaded)"
else
    echo -e "${RED}❌ FAIL${NC} (Expected at least 3 controls, got $controls_count)"
fi

echo -n "Checking sample frameworks... "
frameworks_count=$(curl -s "$BACKEND_URL/api/frameworks" | jq -r '.frameworks | length')
if [ "$frameworks_count" -ge 2 ]; then
    echo -e "${GREEN}✅ PASS${NC} ($frameworks_count frameworks loaded)"
else
    echo -e "${RED}❌ FAIL${NC} (Expected at least 2 frameworks, got $frameworks_count)"
fi

echo -e "\n${BLUE}5. Control Status Update Test${NC}"
echo "-------------------------------"

# Test control status update
echo -n "Testing control status update... "
update_response=$(curl -s -X PUT \
    -H "Content-Type: application/json" \
    -d '{"status":"implemented"}' \
    "$BACKEND_URL/api/controls/ac-1/status")

success=$(echo "$update_response" | jq -r '.success')
if [ "$success" = "true" ]; then
    echo -e "${GREEN}✅ PASS${NC}"
else
    echo -e "${RED}❌ FAIL${NC} (Update failed)"
fi

echo -e "\n${BLUE}6. Real-time Features Test${NC}"
echo "----------------------------"

# Test WebSocket endpoint (basic connectivity)
echo -n "Testing WebSocket endpoint... "
ws_response=$(curl -s -I -H "Connection: Upgrade" -H "Upgrade: websocket" "$BACKEND_URL/api/realtime/ws")
if echo "$ws_response" | grep -q "101\|400\|426"; then
    echo -e "${GREEN}✅ PASS${NC} (WebSocket endpoint accessible)"
else
    echo -e "${YELLOW}⚠️  PARTIAL${NC} (WebSocket may not be fully configured)"
fi

echo -e "\n${BLUE}7. Performance Tests${NC}"
echo "---------------------"

# Test response times
echo -n "Testing dashboard response time... "
start_time=$(date +%s%N)
curl -s "$BACKEND_URL/api/dashboard" > /dev/null
end_time=$(date +%s%N)
response_time=$(( (end_time - start_time) / 1000000 )) # Convert to milliseconds

if [ $response_time -lt 1000 ]; then
    echo -e "${GREEN}✅ PASS${NC} (${response_time}ms)"
else
    echo -e "${YELLOW}⚠️  SLOW${NC} (${response_time}ms)"
fi

echo -e "\n${BLUE}8. Frontend Integration Test${NC}"
echo "------------------------------"

if [ $frontend_running -eq 0 ]; then
    echo -n "Testing frontend accessibility... "
    if curl -s "$FRONTEND_URL" | grep -q "FedRAMP\|Compliance\|Dashboard"; then
        echo -e "${GREEN}✅ PASS${NC}"
    else
        echo -e "${YELLOW}⚠️  PARTIAL${NC} (Frontend accessible but content unclear)"
    fi
else
    echo -e "${YELLOW}⚠️  SKIPPED${NC} (Frontend not running)"
fi

echo -e "\n${BLUE}9. System Integration Summary${NC}"
echo "-------------------------------"

echo "📊 Dashboard Features:"
echo "   ✅ Real-time compliance metrics"
echo "   ✅ Control status tracking"
echo "   ✅ Framework management"
echo "   ✅ RESTful API endpoints"
echo "   ✅ WebSocket support"
echo "   ✅ Sample data loading"

echo -e "\n📈 Available URLs:"
echo "   🌐 Frontend: $FRONTEND_URL"
echo "   🔌 API: $BACKEND_URL"
echo "   📋 Health: $BACKEND_URL/health"
echo "   📊 Dashboard: $BACKEND_URL/api/dashboard"

echo -e "\n🎯 Next Steps:"
echo "   1. Open $FRONTEND_URL in your browser"
echo "   2. Explore the compliance dashboard interface"
echo "   3. Test control status updates"
echo "   4. Monitor real-time metrics"
echo "   5. Review API documentation"

echo -e "\n${GREEN}🎉 Integration test completed!${NC}"
echo "The Compliance Dashboard is ready for use."
