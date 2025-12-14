use std::str::FromStr;
use anyhow::{anyhow,ensure,bail,Result};

#[derive(Debug)]
enum BinaryTree<T>{
    Leaf(T),
    Node(Option<Box<BinaryTree<T>>>, Option<Box<BinaryTree<T>>>)
}

//the result<result> is for severity - outer signals 'its fucked', inner is just 
//'this particular parse isnt valid'

impl<T> BinaryTree<T> 
where T: Clone + Sized + FromStr,
{
    /// deserialize trees encoded as '(a,b)'
    /// (,)\ must be escaped as \\, \, etc.
    pub fn from_string(s : &str) -> Result<(Self, &str)> {
        let tree = Self::try_parse_node(s)?;
        if let (Ok(t),s) = tree {
            return Ok((t,s));
        }
        else { 
            let tree = Self::try_parse_leaf(s)?;
            if let (Ok(Some(t)),s) = tree {
                return Ok((*t,s));
            }
            else { bail!("Failed to parse and we don't know why") }
        }
    }

    //must start with ( and end with ), and contain a , 
    fn try_parse_node(mut s : &str) -> Result<(Result<BinaryTree<T>>, &str)>{
        if s.len() == 0 {return Ok((Ok(BinaryTree::Node(None, None)), &s));}
        if !s.starts_with('(') {
            return Ok((Err(anyhow!("Expected str to start with ( : {}",s)),s));
        }
        
        s = &s[1..s.len()];
        dbg!(s);

        //try and parse the left branch
        let left = Self::try_parse_leaf(s)?;
        let l_node;
        if let (Ok(t),pstr) = left {
            s = pstr;
            l_node = t;
        }
        else {
            let left = Self::try_parse_node(s)?;
            if let (Ok(t),pstr) = left {
                s = pstr;
                l_node = Some(Box::new(t));
            }
            else { bail!("Failed to parse left as node or leaf : {}",s);}
        }
        //make sure the part immediately after the left branch is a comma
        dbg!(s);
        s = Self::try_parse_comma(s)?;
        dbg!(s);

        //try and parse the right branch
        let right = Self::try_parse_leaf(s)?;
        let r_node;
        if let (Ok(t),pstr) = right {
            s = pstr;
            r_node = t;
        }
        else {
            let right = Self::try_parse_node(s)?;
            if let (Ok(t),pstr) = right {
                s = pstr;
                r_node = Some(Box::new(t));
            }
            else { bail!("Failed to parse right as node or leaf : {}",s);}
        }
        dbg!(s);
        if !s.ends_with(')') {
            return Ok((Err(anyhow!("Expected str to end with ) : {s}")),s));
        }
        s = &s[1..];
        dbg!(s);
        return Ok((Ok(BinaryTree::Node(l_node,r_node)), s));
    }

    fn try_parse_comma(s : &str) -> Result<&str>{
        if s.starts_with(",") {Ok(&s[1..])}
        else { bail!("Expected comma : {}", s); }
    }

    //cannot start with ( -> Ok(Err). 
    //if it starts with , its a None leaf
    //terminates at first occurance of unescaped (,)
    //error if we have a valid leaf text but T fails to parse from it.
    fn try_parse_leaf(mut s : &str) -> Result<(Result<Option<Box<BinaryTree<T>>>>,&str)>{
        if s.starts_with("("){return Ok((Err(anyhow!("Unexpected ( starting leaf: {s}")), s));}
        if s.starts_with(","){return Ok((Ok(None),&s[1..]));}
        if s.starts_with(")"){return Ok((Err(anyhow!("Empty nodes not allowed : {s}")), s));}
        if s.len() == 0 {return Err(anyhow!("Unexpected EOF"))};
        let mut prev = &s[..0];
        let mut sub;
        for i in 0..=s.len() {
            sub = &s[0..i];
            if prev.ends_with("\\") { prev = sub; continue; }
            if sub.ends_with(",") || sub.ends_with(")") {
                s=&s[i-1..];
                break;
            }
            if i==s.len() { prev = sub; s = &s[i..]; break; }
            prev = sub;
        }
        let t : T = T::from_str(prev).map_err(|_| 
                                        anyhow!("failed to parse {} as T", prev))?;
        return Ok((Ok(Some(Box::new(BinaryTree::Leaf(t)))),s ));
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use bit_vec::BitVec;

    //58% on notes, 69.6% on bikes, 18% on smol.txt
    #[test]
    fn basic()
    {
        let s = "((a,(d,e)),(bee,c))";
        let tree=BinaryTree::<String>::from_string(s).unwrap();
        dbg!(tree);
    }
}