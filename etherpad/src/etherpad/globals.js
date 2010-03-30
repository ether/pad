/**
 * Copyright 2009 Google Inc.
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

//----------------------------------------------------------------
// global variabls
//----------------------------------------------------------------

var COMETPATH = "/comet";

var COLOR_PALETTE = ['#ffc7c7','#fff1c7','#e3ffc7','#c7ffd5','#c7ffff','#c7d5ff','#e3c7ff','#ffc7f1','#ff8f8f','#ffe38f','#c7ff8f','#8fffab','#8fffff','#8fabff','#c78fff','#ff8fe3','#d97979','#d9c179','#a9d979','#79d991','#79d9d9','#7991d9','#a979d9','#d979c1','#d9a9a9','#d9cda9','#c1d9a9','#a9d9b5','#a9d9d9','#a9b5d9','#c1a9d9','#d9a9cd'];

function isProduction() {
  return (appjet.config['etherpad.isProduction'] == "true");
}

var SUPERDOMAINS = {
  'localbox.info': true,
  'localhost': true,
  'etherpad.com': true
};

var PNE_RELEASE_VERSION = "1.1.3";
var PNE_RELEASE_DATE = "June 15, 2009";

var PRO_FREE_ACCOUNTS = 1e9;


