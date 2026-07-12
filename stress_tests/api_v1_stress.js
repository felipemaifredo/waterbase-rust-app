import http from "k6/http";
import { check, sleep } from "k6";

// Configurações do teste de estresse
export const options = {
  stages: [
    { duration: "5s", target: 10 },  // Ramp-up inicial
    { duration: "10s", target: 30 },  // Ramp-up intermediário
    { duration: "10s", target: 100 },  // Pico: sobe para 500 usuários simultâneos
    { duration: "30s", target: 100 },  // Sustenta 500 usuários por 1 minuto
    { duration: "5s", target: 0 }     // Ramp-down rápido para 0
  ]
}

const BASE_URL = __ENV.BASE_URL || "http://localhost:8080/api/v1";
const API_KEY = __ENV.API_KEY || "waterbase_secret_token_123";

const params = {
  headers: {
    "Authorization": `Bearer ${API_KEY}`,
    "Content-Type": "application/json"
  }
};

export default function () {
  // Agrupa os VUs em 10 coleções no total para ter mais documentos por coleção
  const colName = `stress_col_${__VU % 10}`
  // Inclui o __VU no docId para evitar colisões entre VUs compartilhando a mesma coleção
  const docId = `doc_${__VU}_${__ITER}`

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

  // 8. Auth Flow: register, login, revalidate, logout
  const testEmail = `user_stress_${__VU}_${__ITER}@example.com`
  const testName = `User stress ${__VU} ${__ITER}`

  // 8.1 Register
  const registerPayload = JSON.stringify({
    email: testEmail,
    password: "super_secure_pass_123",
    name: testName,
    document: `doc-stress-${__VU}-${__ITER}`,
    address: `Street stress ${__VU}-${__ITER}, 123`
  })

  let authRes = http.post(`${BASE_URL}/auth/register`, registerPayload, params)
  check(authRes, {
    "POST /auth/register status is 201": (r) => r.status === 201
  })

  const registeredId = authRes.json().id

  // 8.2 Login
  const loginPayload = JSON.stringify({
    email: testEmail,
    password: "super_secure_pass_123"
  })

  authRes = http.post(`${BASE_URL}/auth/login`, loginPayload, params)
  check(authRes, {
    "POST /auth/login status is 200": (r) => r.status === 200,
    "login returns id": (r) => r.json().id === registeredId
  })

  // 8.2.1 Consultar o perfil do usuário diretamente na coleção "users"
  const userProfileRes = http.get(`${BASE_URL}/users/${registeredId}`, params)
  check(userProfileRes, {
    "GET /users/{id} status is 200": (r) => r.status === 200,
    "user profile matches name": (r) => r.json().name === testName
  })

  // 8.2.2 Consultar os dados de autenticação e garantir que password_hash é omitido
  const authDocRes = http.get(`${BASE_URL}/authentication/${registeredId}`, params)
  check(authDocRes, {
    "GET /authentication/{id} status is 200": (r) => r.status === 200,
    "auth doc omits password_hash": (r) => r.json().password_hash === undefined
  })

  const sessionToken = authRes.json().token

  if (sessionToken) {
    const sessionHeader = {
      headers: Object.assign({}, params.headers, {
        "X-Session-Token": sessionToken
      })
    }

    // 8.3 Revalidate
    const revalRes = http.get(`${BASE_URL}/auth/revalidate`, sessionHeader)
    check(revalRes, {
      "GET /auth/revalidate status is 200": (r) => r.status === 200,
      "revalidate returns id": (r) => r.json().id === registeredId
    })

    const nextToken = revalRes.json().token || sessionToken

    const logoutHeader = {
      headers: Object.assign({}, params.headers, {
        "X-Session-Token": nextToken
      })
    }

    // 8.4 Logout
    const logoutRes = http.post(`${BASE_URL}/auth/logout`, null, logoutHeader)
    check(logoutRes, {
      "POST /auth/logout status is 200": (r) => r.status === 200,
      "logout returns success": (r) => r.json().success === true
    })
  }
  // Espera curta entre iterações para simular comportamento real
  sleep(0.1)
}
