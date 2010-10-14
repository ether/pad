import("etherpad.log");
import("dispatch.{Dispatcher,PrefixMatcher,forward}");
import("sqlbase.sqlobj");
import("etherpad.utils");

function queryToSql(args) {
  return [function (querySql) {
    if (request.params.url == undefined || request.params.url == '') {
      return querySql;
    } else  {

      var sql = '' +
       '(select distinct subq.ID from ' +
       '  ' + querySql.sql + ' as subq ' +
       '  join PAD_URL as u on ' +
       '   subq.ID =  u.PAD_ID and ' +
       '   u.URL like ?) ';
      return {
       sql: sql,
       params: querySql.params.concat(['%' + request.params.url + '%'])
      };
    }
  }];
}

function querySummary() {
  var res = utils.renderTemplateAsString("findCitationsQuerySummary.ejs", {}, ['findCitations']);
  if (res.replace(new RegExp("^[ \n]*"), "").replace(new RegExp("[ \n]*$"), "") == '')
    return [];
  return [res];
}

function queryRefiner() {
 return [utils.renderTemplateAsString("findCitationsQueryRefiner.ejs", {}, ['findCitations'])];
}
