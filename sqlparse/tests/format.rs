use sqlparse::{FormatOption, format};


#[test]
fn test_aligned_basic() {
    let sql = r#"
    select a, b as bb,c from table
    join (select a * 2 as a from new_table) other
    on table.a = other.a
    where c is true
    and b between 3 and 4
    or d is 'blue'
    limit 10        
    "#;
    let mut formatter = FormatOption::default();
    formatter.reindent_aligned = true;
    let formatted_sql = format(sql, &mut formatter);
    // println!("{}", formatted_sql);
    assert_eq!(formatted_sql, vec![
        "select a,",
        "       b as bb,",
        "       c",
        "  from table",
        "  join (",
        "        select a * 2 as a",
        "          from new_table",
        "       ) other",
        "    on table.a = other.a",
        " where c is true",
        "   and b between 3 and 4",
        "    or d is 'blue'",
        " limit 10"
    ].join("\n"));
}

#[test]
fn test_aligned_joins() {
    let sql = r#"
    select * from a
    join b on a.one = b.one
    left join c on c.two = a.two and c.three = a.three
    full outer join d on d.three = a.three
    cross join e on e.four = a.four
    join f using (one, two, three)    
    "#;
    let mut formatter = FormatOption::default();
    formatter.reindent_aligned = true;
    let formatted_sql = format(sql, &mut formatter);
    // println!("{}", formatted_sql);
    assert_eq!(formatted_sql, vec![
        "select *",
        "  from a",
        "  join b",
        "    on a.one = b.one",
        "  left join c",
        "    on c.two = a.two",
        "   and c.three = a.three",
        "  full outer join d",
        "    on d.three = a.three",
        " cross join e",
        "    on e.four = a.four",
        "  join f using (one, two, three)"
    ].join("\n"));
}

#[test]
fn test_aligned_case_statement() {
    let sql = r#"
    select a,
    case when a = 0
    then 1
    when bb = 1 then 1
    when c = 2 then 2
    else 0 end as d,
    extra_col
    from table
    where c is true
    and b between 3 and 4    
    "#;
    let mut formatter = FormatOption::default();
    formatter.reindent_aligned = true;
    let formatted_sql = format(sql, &mut formatter);
    // println!("{}", formatted_sql);
    assert_eq!(formatted_sql, vec![
        "select a,",
        "       case when a = 0  then 1",
        "            when bb = 1 then 1",
        "            when c = 2  then 2",
        "            else 0",
        "             end as d,",
        "       extra_col",
        "  from table",
        " where c is true",
        "   and b between 3 and 4",        
    ].join("\n"));
}

#[test]
fn test_aligned_case_statement_with_between() {
    let sql = r#"
    select a,
    case when a = 0
    then 1
    when bb = 1 then 1
    when c = 2 then 2
    when d between 3 and 5 then 3
    else 0 end as d,
    extra_col
    from table
    where c is true
    and b between 3 and 4    
    "#;
    let mut formatter = FormatOption::default();
    formatter.reindent_aligned = true;
    let formatted_sql = format(sql, &mut formatter);
    assert_eq!(formatted_sql, vec![
        "select a,",
        "       case when a = 0             then 1",
        "            when bb = 1            then 1",
        "            when c = 2             then 2",
        "            when d between 3 and 5 then 3",
        "            else 0",
        "             end as d,",
        "       extra_col",
        "  from table",
        " where c is true",
        "   and b between 3 and 4",        
    ].join("\n"));
}


#[test]
fn test_aligned_group_by() {
    let sql = r#"
    select a, b, c, sum(x) as sum_x, count(y) as cnt_y
    from table
    group by a,b,c
    having sum(x) > 1
    and count(y) > 5
    order by 3,2,1    
    "#;
    let mut formatter = FormatOption::default();
    formatter.reindent_aligned = true;
    let formatted_sql = format(sql, &mut formatter);
    assert_eq!(formatted_sql, vec![
        "select a,",
        "       b,",
        "       c,",
        "       sum(x) as sum_x,",
        "       count(y) as cnt_y",
        "  from table",
        " group by a,",
        "          b,",
        "          c",
        "having sum(x) > 1",
        "   and count(y) > 5",
        " order by 3,",
        "          2,",
        "          1"            
    ].join("\n"));      
}