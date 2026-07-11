import http from "k6/http";
import { check, sleep } from "k6";

// Configurações do teste de estresse
export const options = {
  stages: [
    { duration: "10s", target: 40 },  // Ramp-up: sobe para 20 usuários simultâneos em 10s
    { duration: "30s", target: 100 },  // Estresse: sobe para 50 usuários simultâneos em 30s
    { duration: "20s", target: 200 }, // Pico: sobe para 100 usuários simultâneos em 20s
    { duration: "30s", target: 200 }, // Sustenta 100 usuários simultâneos por 30s
    { duration: "15s", target: 0 }    // Ramp-down: reduz para 0 usuários em 15s
  ]
};

const BASE_URL = __ENV.BASE_URL || "http://127.0.0.1:8080/api/v1";
const API_KEY = __ENV.API_KEY || "waterbase_secret_token_123";

const params = {
  headers: {
    "Authorization": `Bearer ${API_KEY}`,
    "Content-Type": "application/json"
  }
};

export default function () {
  const colName = `stress_col_${__VU}`
  const docId = `doc_${__ITER}`

  // 0. GET /health - Verificar saúde do servidor (endpoint público)
  const healthUrl = BASE_URL.replace("/api/v1", "/health")
  const healthRes = http.get(healthUrl)
  check(healthRes, {
    "GET /health status is 200": (r) => r.status === 200,
    "health status is ok": (r) => r.json().status === "ok"
  })

  // 1. GET /collections - Listar collections
  let res = http.get(`${BASE_URL}/collections`, params)
  check(res, {
    "GET /collections status is 200": (r) => r.status === 200
  })

  // 2. POST /{col}/{doc} - Criar documento com timestamp=true
  const createPayload = JSON.stringify({
    name: "Stress Tester",
    vu: __VU,
    iteration: __ITER,
    active: true,
    score: Math.random() * 100
  })
  
  res = http.post(`${BASE_URL}/${colName}/${docId}?timestamp=true`, createPayload, params)
  check(res, {
    "POST /{col}/{doc} status is 201": (r) => r.status === 201
  })

  // 3. GET /{col}/{doc} - Obter documento criado e validar _created_at
  res = http.get(`${BASE_URL}/${colName}/${docId}`, params)
  check(res, {
    "GET /{col}/{doc} status is 200": (r) => r.status === 200,
    "document has correct name": (r) => r.json().name === "Stress Tester",
    "document has _created_at": (r) => typeof r.json()._created_at === "number"
  })

  // 4. GET /{col} - Listar com paginação (limit e offset)
  res = http.get(`${BASE_URL}/${colName}?limit=1&offset=0`, params)
  check(res, {
    "GET /{col} status is 200": (r) => r.status === 200,
    "list pagination works": (r) => r.json().length === 1
  })

  // 5. POST /{col}/_query - Executar query na collection com offset
  const queryPayload = JSON.stringify({
    where: [
      {
        field: "vu",
        op: "==",
        value: __VU
      }
    ],
    offset: 0,
    limit: 10
  })

  res = http.post(`${BASE_URL}/${colName}/_query`, queryPayload, params)
  check(res, {
    "POST /{col}/_query status is 200": (r) => r.status === 200,
    "query returns matches": (r) => r.json().length > 0
  })

  // 6. PUT /{col}/{doc} - Atualizar o documento e validar _updated_at
  const updatePayload = JSON.stringify({
    score: 99.9,
    active: false,
    updated: true
  })

  res = http.put(`${BASE_URL}/${colName}/${docId}`, updatePayload, params)
  check(res, {
    "PUT /{col}/{doc} status is 200": (r) => r.status === 200
  })

  const getUpdated = http.get(`${BASE_URL}/${colName}/${docId}`, params)
  check(getUpdated, {
    "GET updated doc status is 200": (r) => r.status === 200,
    "document has _updated_at": (r) => typeof r.json()._updated_at === "number"
  })

  // 7. DELETE /{col}/{doc} - Deletar o documento para limpar o banco
  res = http.del(`${BASE_URL}/${colName}/${docId}`, null, params)
  check(res, {
    "DELETE /{col}/{doc} status is 200": (r) => r.status === 200
  })

  // Espera curta entre iterações para simular comportamento real
  sleep(0.1)
}
