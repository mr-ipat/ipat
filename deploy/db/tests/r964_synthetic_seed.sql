-- ONLY used by opt-in R9.64 disposable PostgreSQL Docker proof.
-- All IDs/operators/hosts here are explicitly synthetic and globally invalid.
INSERT INTO ipat_platform.tenants(id,tenant_slug) VALUES
 ('11111111-1111-4111-8111-111111111111','tenant-alpha'),
 ('22222222-2222-4222-8222-222222222222','tenant-beta');
INSERT INTO ipat_platform.identity_memberships(
 tenant_id,issuer,subject,role,approved_by,expires_at) VALUES
 ('11111111-1111-4111-8111-111111111111','https://id.synthetic.test.invalid/realms/ipat-lab','synthetic-pilot-operator','tenant_admin','synthetic-approved',now()+interval '1 day'),
 ('22222222-2222-4222-8222-222222222222','https://synthetic.registry.invalid/tenant-b','tenant-b-owner','tenant_admin','synthetic-approved',now()+interval '1 day'),
 ('11111111-1111-4111-8111-111111111111','https://synthetic.r77.invalid/realms/demo','signed-test-operator','noc_engineer','synthetic-approved',now()+interval '1 day');
INSERT INTO ipat_platform.identity_pop_grants(tenant_id,issuer,subject,role,pop_id) VALUES
 ('11111111-1111-4111-8111-111111111111','https://synthetic.r77.invalid/realms/demo','signed-test-operator','noc_engineer','pop-a');
INSERT INTO ipat_ops.managed_devices(
 tenant_id,id,request_id,pop_id,display_name,device_kind,vendor,
 intended_model,management_transport,management_host,management_port,
 added_by_issuer,added_by_subject) VALUES
 ('11111111-1111-4111-8111-111111111111','77777777-7777-4777-8777-777777777771',
 '88888888-8888-4888-8888-888888888881','pop-a','Synthetic existing OLT A','olt','ZTE','C320','ssh','olt-a.fixture.invalid',22,
 'https://id.synthetic.test.invalid/realms/ipat-lab','synthetic-pilot-operator'),
 ('22222222-2222-4222-8222-222222222222','77777777-7777-4777-8777-777777777772',
 '88888888-8888-4888-8888-888888888882','pop-a','Synthetic existing OLT B','olt','ZTE','C320','ssh','olt-b.fixture.invalid',22,
 'https://synthetic.registry.invalid/tenant-b','tenant-b-owner');
