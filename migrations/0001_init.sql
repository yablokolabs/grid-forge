create extension if not exists "uuid-ossp";
create extension if not exists vector;

create table if not exists organizations (
  id uuid primary key,
  name text not null,
  utility_type text not null,
  service_territory text not null,
  demo_mode boolean not null default false,
  retention_days integer not null default 365,
  created_at timestamptz not null default now()
);

create table if not exists users (
  id uuid primary key,
  organization_id uuid not null references organizations(id),
  email text not null,
  display_name text not null,
  role text not null,
  active boolean not null default true,
  created_at timestamptz not null default now(),
  unique (organization_id, email)
);

create table if not exists assets (
  id uuid primary key,
  organization_id uuid not null references organizations(id),
  external_id text not null,
  asset_type text not null,
  name text not null,
  feeder_id text,
  latitude double precision,
  longitude double precision,
  health_index real,
  risk_notes text[] not null default '{}',
  metadata jsonb not null default '{}',
  unique (organization_id, external_id)
);

create table if not exists work_orders (
  id uuid primary key,
  organization_id uuid not null references organizations(id),
  external_id text not null,
  asset_id uuid references assets(id),
  title text not null,
  status text not null,
  priority text not null,
  assigned_crew text,
  notes text not null default '',
  created_at timestamptz not null default now(),
  due_at timestamptz,
  unique (organization_id, external_id)
);

create table if not exists outage_events (
  id uuid primary key,
  organization_id uuid not null references organizations(id),
  outage_number text not null,
  feeder_id text,
  affected_customers integer not null,
  started_at timestamptz not null,
  estimated_restore_at timestamptz,
  cause text,
  status text not null,
  crew_status text,
  unique (organization_id, outage_number)
);

create table if not exists meter_alerts (
  id uuid primary key,
  organization_id uuid not null references organizations(id),
  meter_id text not null,
  account_id uuid,
  alert_type text not null,
  severity text not null,
  observed_at timestamptz not null,
  payload jsonb not null default '{}'
);

create table if not exists documents (
  id uuid primary key,
  organization_id uuid not null references organizations(id),
  title text not null,
  source_uri text not null,
  document_type text not null,
  status text not null,
  tags text[] not null default '{}',
  uploaded_by uuid references users(id),
  created_at timestamptz not null default now()
);

create table if not exists document_chunks (
  id uuid primary key,
  organization_id uuid not null references organizations(id),
  document_id uuid not null references documents(id) on delete cascade,
  chunk_index integer not null,
  text text not null,
  token_count integer not null,
  section text,
  page_start integer,
  page_end integer,
  embedding_model text,
  embedding vector(384),
  unique (document_id, chunk_index)
);

create table if not exists regulations (
  id uuid primary key,
  organization_id uuid not null references organizations(id),
  jurisdiction text not null,
  docket_number text,
  title text not null,
  effective_date timestamptz,
  summary text not null
);

create table if not exists filings (
  id uuid primary key,
  organization_id uuid not null references organizations(id),
  regulation_id uuid references regulations(id),
  docket_number text not null,
  title text not null,
  filed_at timestamptz,
  status text not null,
  document_id uuid references documents(id)
);

create table if not exists customer_interactions (
  id uuid primary key,
  organization_id uuid not null references organizations(id),
  account_id uuid,
  channel text not null,
  raw_text text not null,
  received_at timestamptz not null,
  classification text,
  pii_detected boolean not null default false
);

create table if not exists prompt_templates (
  id uuid primary key,
  organization_id uuid not null references organizations(id),
  module text not null,
  name text not null,
  template text not null,
  requires_human_approval boolean not null default false,
  version integer not null default 1
);

create table if not exists agent_runs (
  id uuid primary key,
  organization_id uuid not null references organizations(id),
  requested_by uuid references users(id),
  module text not null,
  prompt_template_id uuid references prompt_templates(id),
  status text not null,
  approval_state text not null,
  input jsonb not null,
  output jsonb,
  created_at timestamptz not null default now(),
  completed_at timestamptz
);

create table if not exists connectors (
  id uuid primary key,
  organization_id uuid not null references organizations(id),
  name text not null,
  kind text not null,
  status text not null,
  config jsonb not null default '{}',
  created_at timestamptz not null default now()
);

create table if not exists audit_events (
  id uuid primary key,
  organization_id uuid not null references organizations(id),
  actor_user_id uuid references users(id),
  action text not null,
  resource_type text not null,
  resource_id uuid,
  module text,
  citations jsonb not null default '[]',
  decision text,
  metadata jsonb not null default '{}',
  created_at timestamptz not null default now()
);

create index if not exists idx_document_chunks_org on document_chunks (organization_id);
create index if not exists idx_document_chunks_embedding on document_chunks using ivfflat (embedding vector_cosine_ops) with (lists = 100);
create index if not exists idx_audit_events_org_created on audit_events (organization_id, created_at desc);
create index if not exists idx_agent_runs_org_created on agent_runs (organization_id, created_at desc);
