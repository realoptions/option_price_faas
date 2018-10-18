import {url} from './aws'
import {
    UPDATE_CATALOG, 
    SUBSCRIPTION_ERROR,
    CATALOG_ERROR,
    UPDATE_USAGE
} from '../actions/constants'
import {keys} from '../reducers/catalog'
const containsString=(match, string)=>string.toLowerCase().includes(match)
const checkKey=keys=>name=>keys.find(key=>containsString(name, key))
const convertJson=res=>res.json()
export const getCatalog=dispatch=>fetch(`${url}/catalog`)
.then(convertJson)
.then(({items})=>{
    console.log(items)
    const value=items.reduce((aggr, curr)=>{
        const key=checkKey(keys, curr.name)
        if(key){
            return {...aggr, [key]:curr}
        }
        else {
            return aggr
        }
    }, {})
    return dispatch({type:UPDATE_CATALOG, value})
})
//.catch(err=>dispatch({type:CATALOG_ERROR, err}))

export const registerFree=(usagePlanId, client)=>client.invokeApi(
    {},
    `/subscriptions/${usagePlanId}`,
    'PUT',
    {}, {}
)

const marketPlaceSubscribe=(
    usagePlanId, token, client
)=>client.invokeApi(
    {},
    `/marketplace-subscriptions/${usagePlanId}`, 
    'PUT',
    {}, {token}
)

export const registerPaid=(paidUsagePlanId, freeUsagePlanId, token, client)=>Promise.all([
        removeSubscription(freeUsagePlanId, client),
        marketPlaceSubscribe(paidUsagePlanId, token, client)
    ]).then(data=>console.log(data))

export const unregisterPaid=(paidUsagePlanId, freeUsagePlanId, token, client)=>Promise.all([
        removeSubscription(paidUsagePlanId, client),
        registerFree(freeUsagePlanId, client)
    ]).then(data=>console.log(data))


export const removeSubscription=(
    usagePlanId, client
)=>client.invokeApi(
    {},
    `/subscriptions/${usagePlanId}`, 
    'DELETE',
    {}, {}
)

export const getUsage=dispatch=>(
    usagePlanId, client
)=>client.invokeApi(
    {},
    `/subscriptions/${usagePlanId}/usage`,
    'GET', 
    {}, {}
)
.then(({data})=>dispatch({type:UPDATE_USAGE, value:data}))
.catch(err=>dispatch({type:SUBSCRIPTION_ERROR, err}))