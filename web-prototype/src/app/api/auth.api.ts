import {Injectable} from '@angular/core';
import {Apollo} from 'apollo-angular';
import {map} from 'rxjs/operators';
import {DocumentNode, gql} from '@apollo/client/core';
import {Observable, of, Subject} from 'rxjs';
import {GetAccessTokenResponse, UserInterface} from '../interface/auth.interface';
import {HttpClient} from '@angular/common/http';
import {userResponse} from './mock.user';
import {Router} from '@angular/router';
@Injectable()
export class AuthApi {
    private user: UserInterface;
    private userChanges$: Subject<UserInterface> = new Subject<UserInterface>();
    constructor(private apollo: Apollo, private httpClient: HttpClient, private router: Router) {}


    public get onUserChanges(): Observable<UserInterface> {
       return this.userChanges$.asObservable();
    }
    public userChange(user: UserInterface) {
        this.user = user;
        this.userChanges$.next(user);
    }

    public get userModal() {
        return this.user;
    }

    public getAccessToken(code: string): Observable<GetAccessTokenResponse> {
        // const GET_DATA: DocumentNode = gql``;

        debugger;
        return of({access_token: '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'});
        // // @ts-ignore
        // return this.apollo.watchQuery({query: GET_DATA})
        //     .valueChanges.pipe(map((result: any) => {
        //         if (result.data) {
        //             return result.data;
        //         }
        //         return {access_token: 'access_token'};
        //     }));
    }


    public getUserRequest(token: string): Observable<UserInterface> {

        this.userChange(userResponse);
        return of(userResponse);
        // return this.httpClient.get('https://api.github.com/user', {headers: {token: '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'}})
        //     .pipe((map((result: any) => {
        //         this.setUserModal(result);
        //         return result;
        //     })));
    }

    public getUser(token: string = ''): void {
        debugger
      const localStorageAccessToken: string | null = localStorage.getItem('access_token');
      const accessToken: string = (token === '' && localStorageAccessToken) ? localStorageAccessToken : token;

      this.getUserRequest(accessToken).subscribe((user: UserInterface) => {
          localStorage.setItem('access_token', accessToken);
          this.router.navigate(['/']);
      });
  }
}
