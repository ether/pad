/**
 * Copyright 2009 RedHog, Egil Möller <egil.moller@piratpartiet.se>.
 * 
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 * 
 *      http://www.apache.org/licenses/LICENSE-2.0
 * 
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS-IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

import("sqlbase.sqlobj");
import("sqlbase.sqlcommon");

function run() {
  sqlobj.createTable('TAG', {
    ID: 'int not null '+sqlcommon.autoIncrementClause()+' primary key',
    NAME: 'varchar(128) character set utf8 collate utf8_bin not null',
   });

  sqlobj.createTable('PAD_TAG', {
    PAD_ID: 'varchar(128) character set utf8 collate utf8_bin not null references PAD_META(ID)',
    TAG_ID: 'int default NULL references TAG(ID)',
   });

  sqlobj.createTable('PAD_TAG_CACHE', {
    PAD_ID: 'varchar(128) character set utf8 collate utf8_bin unique not null references PAD_META(ID)',
    TAGS: 'varchar(1024) collate utf8_bin not null',
   });
}
