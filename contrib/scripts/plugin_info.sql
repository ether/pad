create view plugin_info as
 select
  p.name as plugin,
  h.name as hook,
  ph.original_name,
  ht.name as type
 from
  plugin as p
  join plugin_hook as ph on
   p.id = ph.plugin_id 
  join hook as h on
   ph.hook_id = h.id 
  join hook_type as ht on
   h.type_id = ht.id;
