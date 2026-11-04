# Sprout API — to-do list de aprendizado (Rust + Axum)

Plano para construir, em ordem de complexidade, a API que o [sprout-web](../sprout-web)
consome. Hoje o frontend usa dados de exemplo em `app/[lang]/data.ts`; a API deve
servir esse mesmo modelo de dados e, em fases posteriores, absorver o "planning
model" (cálculos de metas, plano do mês e fluxo dos próximos 30 dias).

Marque `[x]` conforme completa. Cada item tem **conceitos** (o que você aprende em
Rust/Axum) e **aceite** (como saber que está pronto).

## Contrato geral

- Base: `/scopes/{scope}` onde `scope ∈ {personal, household, company}`.
- Erros: um único formato em toda a API (sugestão: `{ "error": { "code", "message" } }`),
  sempre com status HTTP correto (400/404/409). RFC 9457 (`application/problem+json`)
  é uma alternativa fina.
- Dinheiro: valores inteiros (`i64`), igual ao frontend, que já usa inteiros.
  Só `Holding.change` é flutuante (`f64`).
- Datas de mês: string `YYYY-MM` (ex.: `2025-11`), como o frontend serializa hoje.
- Tipos a espelhar de `data.ts`: `ScopeData`, `Goal` (`split` é opcional),
  `Entry` (`kind: debt | credit | invest`, `confirmed` opcional), `Holding`,
  `Member` (opcional), `CashCycle` (opcional).

---

## Fase 0 — Fundações (sem banco)

### [ ] 0.1 Modelar o domínio em Rust
Estruturas em `src/domain.rs` com derive `Serialize, Deserialize`, `Option` para os
campos que o TS marca com `?` (`split`, `members`, `cashCycle`, `confirmed`) e enums
`Scope` e `EntryKind`.

- **Conceitos:** structs, enums, `Option`, derive, traits `Serialize/Deserialize`, lifetimes desnecessários neste nível (boa notícia).
- **Aceite:**
  - As 3 amostras de `data.ts` desserializam para seus tipos sem erro (teste unitário com o JSON colado em `tests/fixtures/`).
  - Serializar de volta e comparar com o JSON original round-trips sem mudança de valores.

### [ ] 0.2 Formalizar `GET /health`
Já existe quase pronto em `src/api/routes.rs`; mover handler para `src/api/impl.rs`
(arquivo vazio parece ter sido criado pra isso).

- **Conceitos:** handlers async, extrator `Json`, resposta `impl IntoResponse`.
- **Aceite:** `curl localhost:3000/health → 200 {"status":"ok"}` e o código não
  contém closures de handler (handlers nomeados em funções).

### [ ] 0.3 State compartilhado
Criar `AppState` com os dados seed (as 3 scopes) e passá-lo via `State<AppState>`.

- **Conceitos:** `Arc`, `Clone` no state, injeção via `Router::with_state`.
- **Aceite:** o router é construído com estado e compila sem `unwrap` novo no `main.rs`.

---

## Fase 1 — Somente leitura (dados em memória)

### [ ] 1.1 `GET /scopes`
Lista os escopos disponíveis. Ids fixos por enquanto.

- **Conceitos:** roteamento básico, retornar `Vec<&Scope>`.
- **Aceite:** responde `[{"id":"personal",...}]`; ids estão na UI do frontend
  (`scopes` em `data.ts`), sem hardcode espalhado (uma única constante).

### [ ] 1.2 `GET /scopes/{scope}`
Retorna o `ScopeData` completo. O frontend pode ser ligado nesse endpoint trocando
`scopeData` por um `fetch` — comportamento idêntico ao mock atual.

- **Conceitos:** path params (`Path<ScopePath>`), validação do enum (`impl FromStr` via `serde`), erro 404 como valor de retorno.
- **Aceite:**
  - JSON de resposta tem as mesmas chaves/valores do mock de `data.ts` (teste de snapshot).
  - `/scopes/foo → 404` com o formato de erro do contrato.

### [ ] 1.3 `GET /scopes/{scope}/goals/{id}/progress`
Porta `goalProgress()` de `data.ts` para Rust: history, pace (média dos últimos 3),
plan, milestones, projeção, chegada/atraso, skip-next-month.

- **Conceitos:** cálculo puro em função livre com testes; fatiamento de `Vec`,
  `iter`, `zip`, `map`, retorno de enums em vez de sentinelas (`Option`, `Result`,
  `Infinity` do TS vira `Option` — aqui está seu primeiro ganho real de modelagem).
- **Aceite:**
  - Testes espelham os casos de `data.test.ts` do frontend; mesmos números para os mesmos inputs.
  - `arrival`/`skipArrival` nulos quando o pace é 0 (era `Infinity` no TS).

### [ ] 1.4 `GET /scopes/{scope}/plan`
Porta `monthPlan()`: surplus, soma do neededThisMonth das metas ativas, gap e
quais metas ficam subfinanciadas e quantos meses escorregam.

- **Conceitos:** compor funções do domínio; ordenar por deadline (`sort_by`/`sort_by_key` com `Ord`); evitar `clone` desnecessário → ler `&self`.
- **Aceite:** teste com os dados seed bate com o valor do frontend; metas ativas
  ordenadas por prazo mais próximo primeiro.

### [ ] 1.5 `GET /scopes/{scope}/flow`
Porta `weeklyFlow()`: 4 janelas (dias 1–7, 8–14, 15–21, 22–30) com confirmedIn,
expectedIn e out.

- **Conceitos:** `filter`, `partition_map` ou agregação manual; decisão de representar janelas como struct, de forma explícita.
- **Aceite:** janelas batem com o mock; crédito sem `confirmed` conta como expected, nunca como confirmado.

### [ ] 1.6 `GET /scopes/{scope}/members`
Retorna `members` com `surplus` e `put` (porta `memberShares()`). Para `personal`
e `company`: 404 ou `[]` — decida e documente no README.

- **Conceitos:** `Option` + computação condicional; documentar decisões no README.
- **Aceite:** household retorna as cotas por membro usando os `split` das metas;
  escolha de 404 ou vazio está coberta por teste.

### [ ] 1.7 Testes de integração
Testar as rotas via `axum::Router` com `tower::ServiceExt::oneshot` (sem bindar porta).

- **Conceitos:** `tokio::test`, `oneshot`, builder do app extraído de `main.rs` (função `app()` reutilizada).
- **Aceite:** `cargo test` cobre todas as rotas Fase 1; `main.rs` só sobe o server.

---

## Fase 2 — Persistência + CRUD

Escolha **SQLite + sqlx** para começar (migrações embutidas, um binário só).
Trocar o armazenamento em memória pelo banco mantendo as rotas de leitura da
Fase 1 inalteradas é o teste real de arquitetura aqui.

### [ ] 2.1 Infra de banco
sqlx com SQLite, pool no `AppState`, migração inicial com os dados seed.

- **Conceitos:** async no mundo real, `sqlx::Pool`, migrações, trait de repositório.
- **Aceite:** app sobe em DB vazio e aplica migrações; leituras da fase 1
  respondem igual com dados do banco (fim dos dados hardcoded).

### [ ] 2.2 `CRUD /scopes/{scope}/goals`
`GET` (lista), `POST` (cria), `GET/:id`, `PATCH/:id` (atualiza target/meses/split/início),
`DELETE /:id`.

- **Conceitos:** extractors com corpo (`Json<CreateGoal>`), validação com `Result`, `From<sqlx::Error> para AppError`, trait de repositório para desacoplar handler e banco.
- **Aceite:**
  - `POST` sem `target`/`months`/`start` → 400 com mensagem específica do campo faltando.
  - `GET` de id inexistente → 404; `DELETE` de inexistente → 404; `DELETE` existente → 204.
  - `split` de household que cita id de membro inexistente → 400.

### [ ] 2.3 `POST /scopes/{scope}/goals/{id}/contributions`
Apenda uma contribuição no `history` do mês corrente. É a operação de domínio mais
importante do app (a meta evolui mês a mês).

- **Conceitos:** modelo de domínio com invariantes (contribution não pode reduzir
  abaixo de 0? uma por mês? decida e reforce), transação no sqlx, conflitos → 409.
- **Aceite:** contribuição no mesmo mês duas vezes → 409 (ou a regra que você documentou); entrada correta apenda `history` do mês certo e o `/progress` reflete na hora.

### [ ] 2.4 `CRUD /scopes/{scope}/entries`
Mesmo padrão do goals para os lançamentos futuros (credits, debts, invests), com
`kind` validado e `confirmed` marcar crédito (muda a coluna "confirmedIn" do flow).

- **Conceitos:** enum com `#[serde(rename_all = "lowercase")]`, validação em camada própria, PATCH parcial (`Option<T>` campos "tri-state": ausente = não muda).
- **Aceite:** kind inválido → 400; PATCH sem um campo não altera esse campo; PATCH só com `{ "confirmed": true }` em um crédito move o valor de `expectedIn` para `confirmedIn` na semana certa do flow.

### [ ] 2.5 Net worth e holdings
`GET /scopes/{scope}/networth` (séries) com `POST /networth/snapshots`;
`CRUD /holdings`.

- **Conceitos:** duas collections por scope, relacionamento, migração adicional.
- **Aceite:** snapshot criado aparece no fim da série de `netWorth.history`; leitura retorna os mesmos números do mock.

### [ ] 2.6 Members e cashCycle (household/company)
`GET/PATCH /scopes/{scope}/members`, `PATCH /scopes/{scope}/cash-cycle`.

- **Conceitos:** dados que existem só em parte dos scopes; evite herança — está aprendendo composição.
- **Aceite:** personal não tem members; PATCH em household muda income/expenses e o surplus do `/plan` reflete.

---

## Fase 3 — Qualidade e ergonomia

### [ ] 3.1 CORS para o frontend
`tower-http` `CorsLayer` permitindo a origem do dev (localhost:3000), métodos
headers do dia a dia.

- **Conceitos:** middleware/layers empilhados (ordem importa com `TraceLayer`).
- **Aceite:** página do sprout-web consumindo a API via `fetch` num dev local mostra dado real na Home; preflight `OPTIONS` responde 204/200 sem erro no console.

### [ ] 3.2 Erros centralizados e validação decente
Um `AppError` enum com `IntoResponse` e mapeamento de código HTTP por variante;
validação acumulando todos os problemas de uma request quando possível.

- **Conceitos:** `From`, `?` no handler, enum com dados, propagação de erro de verdade.
- **Aceite:** nenhum handler constrói resposta de erro manualmente; testes cobrem
  cada código de erro (400/404/409) em uma rota de CRUD qualquer.

### [ ] 3.3 Config por ambiente
Porta, nível de log, URL do banco via variáveis de ambiente (`SPROUT_BIND`,
`SPROUT_DATABASE_URL`), com defaults para dev.

- **Conceitos:** nunca `unwrap` no main, `Result` no entrypoint, mudar main para
  `fn main() -> ExitCode` ou `anyhow`.
- **Aceite:** testes de integração não conflitam com dev server na porta 3000.

### [ ] 3.4 Endpoints paginados (se atingir volume)
`GET /entries?limit=&cursor=`, com paginação cursor-based.

- **Conceitos:** query params (`Query<T>`), default e valida limites de página.
- **Aceite:** sem `limit`, usa default; `limit=1000` é clampado ou rejeitado com 400 (nunca crash); pages seguem o cursor sem pular/duplicar item.

---

## Fase 4 — Extras (escolha quando terminar a Fase 3)

### [ ] 4.1 Multi-usuário leve (auth)
Registro/login com senha, sessão via JWT ou cookie; cada usuário tem seus
próprios scopes. Household passa a ter múltiplos integrantes (relação user↔scope).

- **Conceitos:** middleware custom (extrator `FromRequestParts`), hash de senha
  (`argon2`), não inventar crypto.
- **Aceite:** sem token → 401 em tudo exceto `/health` e auth; usuário A não vê
  os scopes do usuário B (teste).

### [ ] 4.2 Docker + deploy
Multi-stage `Dockerfile` (build → runtime distroless/alpine), tamanho e startup rápidos.

- **Conceitos:** build release estático, camadas de cache, `cargo build --release`.
- **Aceite:** `docker run sprout-api` sobe com SQLite e health check passa.

### [ ] 4.3 OpenAPI
Documentar rotas via `utoipa` e servir UI (swagger/redoc) em dev.

- **Conceitos:** documentação gerada a partir dos tipos (derive por struct).
- **Aceite:** schema gerado bate com os handlers (contrato reutilizável pro frontend).

---

## Ordem sugerida de estudo paralelo

1. **Rust básico:** The Book (cap. 1–10) até chegar no item 0.1.
2. **Axum:** exemplos oficiais (`axum/examples`) ao iniciar Fase 1.
3. **sqlx:** docs de `query!`/`query_as!` ao entrar na Fase 2.
4. **Macro de leitura:** a cada item, leia o código do sprout-web que mostra como
   o dado será consumido antes de desenhar a rota — o contrato é o frontend.

## Progresso

- [ ] 0.1 tipos
- [ ] 0.2 health
- [ ] 0.3 estado
- [ ] 1.1–1.7 leitura completa
- [ ] 2.1–2.6 CRUD com banco
- [ ] 3.1–3.4 qualidade
- [ ] 4.x extras
