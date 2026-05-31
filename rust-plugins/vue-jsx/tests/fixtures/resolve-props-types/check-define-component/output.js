function scope() {
    function defineComponent1() {}
    defineComponent1((props: {
        msg: string;
    })=>{});
}
defineComponent((props: {
    msg: string;
})=>{});
