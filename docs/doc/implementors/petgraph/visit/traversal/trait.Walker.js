(function() {var implementors = {};
implementors["daggy"] = [{"text":"impl&lt;'a, G, F&gt; <a class=\"trait\" href=\"daggy/trait.Walker.html\" title=\"trait daggy::Walker\">Walker</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.reference.html\">&amp;'a </a>G&gt; for <a class=\"struct\" href=\"daggy/walker/struct.Recursive.html\" title=\"struct daggy::walker::Recursive\">Recursive</a>&lt;G, F&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;G: <a class=\"trait\" href=\"petgraph/visit/trait.GraphBase.html\" title=\"trait petgraph::visit::GraphBase\">GraphBase</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;F: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/function/trait.FnMut.html\" title=\"trait core::ops::function::FnMut\">FnMut</a>(<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.reference.html\">&amp;</a>G, G::<a class=\"associatedtype\" href=\"petgraph/visit/trait.GraphBase.html#associatedtype.NodeId\" title=\"type petgraph::visit::GraphBase::NodeId\">NodeId</a>) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;(G::<a class=\"associatedtype\" href=\"petgraph/visit/trait.GraphBase.html#associatedtype.EdgeId\" title=\"type petgraph::visit::GraphBase::EdgeId\">EdgeId</a>, G::<a class=\"associatedtype\" href=\"petgraph/visit/trait.GraphBase.html#associatedtype.NodeId\" title=\"type petgraph::visit::GraphBase::NodeId\">NodeId</a>)&gt;,&nbsp;</span>","synthetic":false,"types":["daggy::walker::Recursive"]},{"text":"impl&lt;'a, G, A, B&gt; <a class=\"trait\" href=\"daggy/trait.Walker.html\" title=\"trait daggy::Walker\">Walker</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.reference.html\">&amp;'a </a>G&gt; for <a class=\"struct\" href=\"daggy/walker/struct.Chain.html\" title=\"struct daggy::walker::Chain\">Chain</a>&lt;G, A, B&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;G: <a class=\"trait\" href=\"petgraph/visit/trait.GraphBase.html\" title=\"trait petgraph::visit::GraphBase\">GraphBase</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;A: <a class=\"trait\" href=\"daggy/trait.Walker.html\" title=\"trait daggy::Walker\">Walker</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.reference.html\">&amp;'a </a>G&gt;,<br>&nbsp;&nbsp;&nbsp;&nbsp;B: <a class=\"trait\" href=\"daggy/trait.Walker.html\" title=\"trait daggy::Walker\">Walker</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.reference.html\">&amp;'a </a>G, Item = A::<a class=\"associatedtype\" href=\"daggy/trait.Walker.html#associatedtype.Item\" title=\"type daggy::Walker::Item\">Item</a>&gt;,&nbsp;</span>","synthetic":false,"types":["daggy::walker::Chain"]},{"text":"impl&lt;G, W, P&gt; <a class=\"trait\" href=\"daggy/trait.Walker.html\" title=\"trait daggy::Walker\">Walker</a>&lt;G&gt; for <a class=\"struct\" href=\"daggy/walker/struct.Filter.html\" title=\"struct daggy::walker::Filter\">Filter</a>&lt;G, W, P&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;G: <a class=\"trait\" href=\"petgraph/visit/trait.GraphRef.html\" title=\"trait petgraph::visit::GraphRef\">GraphRef</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;W: <a class=\"trait\" href=\"daggy/trait.Walker.html\" title=\"trait daggy::Walker\">Walker</a>&lt;G&gt;,<br>&nbsp;&nbsp;&nbsp;&nbsp;P: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/function/trait.FnMut.html\" title=\"trait core::ops::function::FnMut\">FnMut</a>(G, &amp;W::<a class=\"associatedtype\" href=\"daggy/trait.Walker.html#associatedtype.Item\" title=\"type daggy::Walker::Item\">Item</a>) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.bool.html\">bool</a>,&nbsp;</span>","synthetic":false,"types":["daggy::walker::Filter"]},{"text":"impl&lt;G, W&gt; <a class=\"trait\" href=\"daggy/trait.Walker.html\" title=\"trait daggy::Walker\">Walker</a>&lt;G&gt; for <a class=\"struct\" href=\"daggy/walker/struct.Peekable.html\" title=\"struct daggy::walker::Peekable\">Peekable</a>&lt;G, W&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;G: <a class=\"trait\" href=\"petgraph/visit/trait.GraphRef.html\" title=\"trait petgraph::visit::GraphRef\">GraphRef</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;W: <a class=\"trait\" href=\"daggy/trait.Walker.html\" title=\"trait daggy::Walker\">Walker</a>&lt;G&gt;,&nbsp;</span>","synthetic":false,"types":["daggy::walker::Peekable"]},{"text":"impl&lt;G, W, P&gt; <a class=\"trait\" href=\"daggy/trait.Walker.html\" title=\"trait daggy::Walker\">Walker</a>&lt;G&gt; for <a class=\"struct\" href=\"daggy/walker/struct.SkipWhile.html\" title=\"struct daggy::walker::SkipWhile\">SkipWhile</a>&lt;G, W, P&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;G: <a class=\"trait\" href=\"petgraph/visit/trait.GraphRef.html\" title=\"trait petgraph::visit::GraphRef\">GraphRef</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;W: <a class=\"trait\" href=\"daggy/trait.Walker.html\" title=\"trait daggy::Walker\">Walker</a>&lt;G&gt;,<br>&nbsp;&nbsp;&nbsp;&nbsp;P: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/function/trait.FnMut.html\" title=\"trait core::ops::function::FnMut\">FnMut</a>(G, &amp;W::<a class=\"associatedtype\" href=\"daggy/trait.Walker.html#associatedtype.Item\" title=\"type daggy::Walker::Item\">Item</a>) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.bool.html\">bool</a>,&nbsp;</span>","synthetic":false,"types":["daggy::walker::SkipWhile"]},{"text":"impl&lt;G, W, P&gt; <a class=\"trait\" href=\"daggy/trait.Walker.html\" title=\"trait daggy::Walker\">Walker</a>&lt;G&gt; for <a class=\"struct\" href=\"daggy/walker/struct.TakeWhile.html\" title=\"struct daggy::walker::TakeWhile\">TakeWhile</a>&lt;G, W, P&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;G: <a class=\"trait\" href=\"petgraph/visit/trait.GraphRef.html\" title=\"trait petgraph::visit::GraphRef\">GraphRef</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;W: <a class=\"trait\" href=\"daggy/trait.Walker.html\" title=\"trait daggy::Walker\">Walker</a>&lt;G&gt;,<br>&nbsp;&nbsp;&nbsp;&nbsp;P: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/function/trait.FnMut.html\" title=\"trait core::ops::function::FnMut\">FnMut</a>(G, &amp;W::<a class=\"associatedtype\" href=\"daggy/trait.Walker.html#associatedtype.Item\" title=\"type daggy::Walker::Item\">Item</a>) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.bool.html\">bool</a>,&nbsp;</span>","synthetic":false,"types":["daggy::walker::TakeWhile"]},{"text":"impl&lt;G, W&gt; <a class=\"trait\" href=\"daggy/trait.Walker.html\" title=\"trait daggy::Walker\">Walker</a>&lt;G&gt; for <a class=\"struct\" href=\"daggy/walker/struct.Skip.html\" title=\"struct daggy::walker::Skip\">Skip</a>&lt;G, W&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;G: <a class=\"trait\" href=\"petgraph/visit/trait.GraphRef.html\" title=\"trait petgraph::visit::GraphRef\">GraphRef</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;W: <a class=\"trait\" href=\"daggy/trait.Walker.html\" title=\"trait daggy::Walker\">Walker</a>&lt;G&gt;,&nbsp;</span>","synthetic":false,"types":["daggy::walker::Skip"]},{"text":"impl&lt;G, W&gt; <a class=\"trait\" href=\"daggy/trait.Walker.html\" title=\"trait daggy::Walker\">Walker</a>&lt;G&gt; for <a class=\"struct\" href=\"daggy/walker/struct.Take.html\" title=\"struct daggy::walker::Take\">Take</a>&lt;G, W&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;G: <a class=\"trait\" href=\"petgraph/visit/trait.GraphRef.html\" title=\"trait petgraph::visit::GraphRef\">GraphRef</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;W: <a class=\"trait\" href=\"daggy/trait.Walker.html\" title=\"trait daggy::Walker\">Walker</a>&lt;G&gt;,&nbsp;</span>","synthetic":false,"types":["daggy::walker::Take"]},{"text":"impl&lt;G, W&gt; <a class=\"trait\" href=\"daggy/trait.Walker.html\" title=\"trait daggy::Walker\">Walker</a>&lt;G&gt; for <a class=\"struct\" href=\"daggy/walker/struct.Cycle.html\" title=\"struct daggy::walker::Cycle\">Cycle</a>&lt;G, W&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;G: <a class=\"trait\" href=\"petgraph/visit/trait.GraphRef.html\" title=\"trait petgraph::visit::GraphRef\">GraphRef</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;W: <a class=\"trait\" href=\"daggy/trait.Walker.html\" title=\"trait daggy::Walker\">Walker</a>&lt;G&gt; + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a>,&nbsp;</span>","synthetic":false,"types":["daggy::walker::Cycle"]},{"text":"impl&lt;G, W, F&gt; <a class=\"trait\" href=\"daggy/trait.Walker.html\" title=\"trait daggy::Walker\">Walker</a>&lt;G&gt; for <a class=\"struct\" href=\"daggy/walker/struct.Inspect.html\" title=\"struct daggy::walker::Inspect\">Inspect</a>&lt;W, F&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;G: <a class=\"trait\" href=\"petgraph/visit/trait.GraphRef.html\" title=\"trait petgraph::visit::GraphRef\">GraphRef</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;W: <a class=\"trait\" href=\"daggy/trait.Walker.html\" title=\"trait daggy::Walker\">Walker</a>&lt;G&gt;,<br>&nbsp;&nbsp;&nbsp;&nbsp;F: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/function/trait.FnMut.html\" title=\"trait core::ops::function::FnMut\">FnMut</a>(G, &amp;W::<a class=\"associatedtype\" href=\"daggy/trait.Walker.html#associatedtype.Item\" title=\"type daggy::Walker::Item\">Item</a>),&nbsp;</span>","synthetic":false,"types":["daggy::walker::Inspect"]},{"text":"impl&lt;'a, N, E, Ix&gt; <a class=\"trait\" href=\"daggy/trait.Walker.html\" title=\"trait daggy::Walker\">Walker</a>&lt;&amp;'a <a class=\"struct\" href=\"daggy/struct.Dag.html\" title=\"struct daggy::Dag\">Dag</a>&lt;N, E, Ix&gt;&gt; for <a class=\"struct\" href=\"daggy/struct.Children.html\" title=\"struct daggy::Children\">Children</a>&lt;N, E, Ix&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;Ix: <a class=\"trait\" href=\"petgraph/graph_impl/trait.IndexType.html\" title=\"trait petgraph::graph_impl::IndexType\">IndexType</a>,&nbsp;</span>","synthetic":false,"types":["daggy::Children"]},{"text":"impl&lt;'a, N, E, Ix&gt; <a class=\"trait\" href=\"daggy/trait.Walker.html\" title=\"trait daggy::Walker\">Walker</a>&lt;&amp;'a <a class=\"struct\" href=\"daggy/struct.Dag.html\" title=\"struct daggy::Dag\">Dag</a>&lt;N, E, Ix&gt;&gt; for <a class=\"struct\" href=\"daggy/struct.Parents.html\" title=\"struct daggy::Parents\">Parents</a>&lt;N, E, Ix&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;Ix: <a class=\"trait\" href=\"petgraph/graph_impl/trait.IndexType.html\" title=\"trait petgraph::graph_impl::IndexType\">IndexType</a>,&nbsp;</span>","synthetic":false,"types":["daggy::Parents"]}];
implementors["petgraph"] = [];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()