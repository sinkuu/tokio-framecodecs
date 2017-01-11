(function() {var implementors = {};
implementors["framecodecs"] = ["impl&lt;T:&nbsp;<a class='trait' href='https://docs.rs/tokio-core/0.1/tokio_core/io/trait.Io.html' title='tokio_core::io::Io'>Io</a> + 'static&gt; ServerProto&lt;T&gt; for <a class='struct' href='framecodecs/fixed_length/struct.FixedLengthProto.html' title='framecodecs::fixed_length::FixedLengthProto'>FixedLengthProto</a>","impl&lt;T, D&gt; ServerProto&lt;T&gt; for <a class='struct' href='framecodecs/delimiter/struct.DelimiterProto.html' title='framecodecs::delimiter::DelimiterProto'>DelimiterProto</a>&lt;D&gt; <span class='where fmt-newline'>where T: <a class='trait' href='https://docs.rs/tokio-core/0.1/tokio_core/io/trait.Io.html' title='tokio_core::io::Io'>Io</a> + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;D: <a class='trait' href='framecodecs/delimiter/trait.Delimiter.html' title='framecodecs::delimiter::Delimiter'>Delimiter</a> + <a class='trait' href='https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html' title='core::clone::Clone'>Clone</a> + 'static</span>","impl&lt;B:&nbsp;ByteOrder + 'static, T:&nbsp;<a class='trait' href='https://docs.rs/tokio-core/0.1/tokio_core/io/trait.Io.html' title='tokio_core::io::Io'>Io</a> + 'static&gt; ServerProto&lt;T&gt; for <a class='struct' href='framecodecs/length_field/struct.LengthFieldProto.html' title='framecodecs::length_field::LengthFieldProto'>LengthFieldProto</a>&lt;B&gt;","impl&lt;C&gt; ServerProto&lt;<a class='struct' href='https://docs.rs/tokio-core/0.1/tokio_core/net/tcp/struct.TcpStream.html' title='tokio_core::net::tcp::TcpStream'>TcpStream</a>&gt; for <a class='struct' href='framecodecs/remote_addr/struct.RemoteAddrProto.html' title='framecodecs::remote_addr::RemoteAddrProto'>RemoteAddrProto</a>&lt;C, Pipeline&gt; <span class='where fmt-newline'>where C: <a class='trait' href='https://docs.rs/tokio-core/0.1/tokio_core/io/frame/trait.Codec.html' title='tokio_core::io::frame::Codec'>Codec</a> + <a class='trait' href='https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html' title='core::clone::Clone'>Clone</a> + 'static</span>","impl&lt;T&gt; ServerProto&lt;T&gt; for <a class='struct' href='framecodecs/varint/struct.VarIntLengthFieldProto.html' title='framecodecs::varint::VarIntLengthFieldProto'>VarIntLengthFieldProto</a> <span class='where'>where T: <a class='trait' href='https://docs.rs/tokio-core/0.1/tokio_core/io/trait.Io.html' title='tokio_core::io::Io'>Io</a> + 'static</span>",];

            if (window.register_implementors) {
                window.register_implementors(implementors);
            } else {
                window.pending_implementors = implementors;
            }
        
})()
