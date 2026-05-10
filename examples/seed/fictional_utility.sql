-- Fictional seed data for Cedar Rapids Light & Power.
-- Safe for local development only.

insert into organizations (id, name, utility_type, service_territory, demo_mode, retention_days)
values ('aaaaaaaa-aaaa-4aaa-aaaa-aaaaaaaaaaaa', 'Cedar Rapids Light & Power', 'municipal', 'Cedar Rapids metro demo territory', true, 365)
on conflict (id) do nothing;

insert into users (id, organization_id, email, display_name, role, active)
values
  ('11111111-1111-4111-8111-111111111111', 'aaaaaaaa-aaaa-4aaa-aaaa-aaaaaaaaaaaa', 'engineer@cedar-rapids.example', 'Demo Engineer', 'utility_engineer', true),
  ('22222222-2222-4222-8222-222222222222', 'aaaaaaaa-aaaa-4aaa-aaaa-aaaaaaaaaaaa', 'regulatory@cedar-rapids.example', 'Demo Regulatory Analyst', 'regulatory_analyst', true),
  ('33333333-3333-4333-8333-333333333333', 'aaaaaaaa-aaaa-4aaa-aaaa-aaaaaaaaaaaa', 'customerops@cedar-rapids.example', 'Demo Customer Ops', 'customer_ops', true),
  ('66666666-6666-4666-8666-666666666666', 'aaaaaaaa-aaaa-4aaa-aaaa-aaaaaaaaaaaa', 'admin@cedar-rapids.example', 'Demo Admin', 'admin', true)
on conflict (id) do nothing;

insert into assets (id, organization_id, external_id, asset_type, name, feeder_id, health_index, risk_notes, metadata)
values
  ('aaaaaaaa-1111-4111-8111-aaaaaaaa1111', 'aaaaaaaa-aaaa-4aaa-aaaa-aaaaaaaaaaaa', 'TX-8841', 'transformer', 'Padmount Transformer TX-8841', 'F-12', 0.72, array['summer peak loading above 92%', 'two related voltage complaints'], '{"kva": 500}'::jsonb),
  ('aaaaaaaa-2222-4222-8222-aaaaaaaa2222', 'aaaaaaaa-aaaa-4aaa-aaaa-aaaaaaaaaaaa', 'F-12', 'feeder', 'Feeder F-12', 'F-12', 0.64, array['vegetation exposure', 'storm outage history'], '{"criticalCustomers": 3}'::jsonb)
on conflict (id) do nothing;

insert into outage_events (id, organization_id, outage_number, feeder_id, affected_customers, started_at, estimated_restore_at, cause, status, crew_status)
values ('bbbbbbbb-1111-4111-8111-bbbbbbbb1111', 'aaaaaaaa-aaaa-4aaa-aaaa-aaaaaaaaaaaa', 'OUT-2026-0417', 'F-12', 184, now() - interval '2 hours', now() + interval '90 minutes', 'suspected vegetation contact', 'crew_assigned', 'crew en route')
on conflict (id) do nothing;
