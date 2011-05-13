/**
 * Copyright 2009 RedHog, Egil Möller <egil.moller@piratpartiet.se>
 * Copyright 2010 Pita, Peter Martischka <petermartischka@googlemail.com>
 * 
 * Some code from http://groups.google.com/group/etherpad-open-source-discuss/msg/5001fe0ef2fac58a
 * Copyright by dannydulai@gmail.com
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

import("faststatic");
import("dispatch.{Dispatcher,PrefixMatcher,forward}");

import("etherpad.utils.*");
import("etherpad.collab.server_utils");
import("etherpad.globals.*");
import("etherpad.log");
import("etherpad.pad.padusers");
import("etherpad.pro.pro_utils");
import("etherpad.pro.domains");
import("etherpad.helpers");
import("etherpad.pro.pro_accounts.getSessionProAccount");
import("sqlbase.sqlbase");
import("sqlbase.sqlcommon");
import("sqlbase.sqlobj");
import("etherpad.pad.padutils");
import("etherpad.admin.plugins");
import("fastJSON");
import("etherpad.pad.model");
import("etherpad.pad.dbwriter");
import("etherpad.collab.collab_server");
import("etherpad.sessions.getSession");

function _isAuthorizedAdmin() {
  if (!isProduction()) {
    return true;
  }
  return (getSession().adminAuth === true);
}

function onRequest() {
  if (!_isAuthorizedAdmin()) {
    getSession().cont = request.path;
    response.redirect('/ep/admin/auth');
  }

  var isPro = pro_utils.isProDomainRequest();
  var userId = padusers.getUserId();

  helpers.addClientVars({
    userAgent: request.headers["User-Agent"],
    debugEnabled: request.params.djs,
    clientIp: request.clientAddr,
    colorPalette: COLOR_PALETTE,
    serverTimestamp: +(new Date),
    isProPad: isPro,
    userIsGuest: padusers.isGuest(userId),
    userId: userId
  });

  var isProUser = (isPro && ! padusers.isGuest(userId));

  if (request.isPost) {
    log.info({'PAD': request.params.pad_id});

    model.accessPadGlobal(request.params.pad_id, function(pad) { 
      collab_server.bootUsersFromPad(pad, "deleted"); 
      pad.destroy(); 
    }); 
    dbwriter.taskFlushPad(request.params.pad_id, "delete"); 
  }

  renderHtml(
    "ui.ejs",
    {
      bodyClass: 'delete-pad',
      isPro: isPro,
      isProAccountHolder: isProUser,
      account: getSessionProAccount() // may be falsy
    },
    ['deletePad']);
  return true;
}
